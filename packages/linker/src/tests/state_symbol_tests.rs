//! The maturity link's symbol census, asserted against the real record.
//!
//! # The figures are read, never restated
//!
//! Every count below is compared with the composed announcement record
//! and the candidate constructor themselves — the shared maturity
//! fixtures in [`crate::tests`], reached through public entry points. A
//! test that wrote the census down from a document would pass on the day
//! the record stopped matching it, which is the one day it needs to fail.
//!
//! # The closed refusal root is covered by test or by reason
//!
//! `the_closed_refusal_root_is_covered_by_test_or_by_reason` walks
//! [`StateLinkRefusal`] through an exhaustive match, so a variant added
//! to it fails to compile here until this file says which test reaches it
//! or why nothing can. The graph's refusals are walked here too, because
//! the root is one census and a second walk beside it would be a second
//! authority on which variants exist. Three are declared unreachable:
//! the revision disagreement, because one reviewed contract revision is
//! constructible and both operands are read from it; the invalid item,
//! because every value the census builds is eight or thirty-two bytes
//! and the reviewed literal bound is far above both; and the frozen
//! graph, because the constructor states seven references against a
//! bound of sixty-four.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ARCHITECTURE, AssetId};
use realization::{STATE_METADATA_LAYOUT, StateMetadataRegionClass};
use tapscript::upstream::{Cycle, StateSingletonDeclaration};
use tapscript::{
    StackItem, StateConstructorReference, StateLeafRole, StateProgramWitness, TapscriptInstruction,
};
use target_elements::{ReviewedElementsTapscriptDefinition, TargetContractVersion};

use crate::tests::state_graph_tests::residual_cycle_refusal;
use crate::tests::{
    bridge, declaration, record, record_for_schedule, reviewed_target, singleton, state_constructor,
};
use crate::{
    STATE_REFERENCE_LIMIT, StateBindingTime, StateConsumerCensus, StateDefinitionCensus,
    StateDefinitionOrigin as Origin, StateGraphNode, StateLinkRefusal, StateLinkSymbol as Key,
    StateReferenceGraphRefusal, StateSymbolType, StateSymbolValue, collect_state_definitions,
    resolve_state_census, state_declared_type,
};

/// Pass one over the demonstration sources.
fn definitions(target: &ReviewedElementsTapscriptDefinition) -> StateDefinitionCensus {
    collect_state_definitions(
        target,
        &bridge(),
        &state_constructor(),
        &singleton(),
        &declaration(),
        tapscript::StateWitnessSchedule::WholeMetadata,
    )
    .expect("the demonstration sources define every key")
}

/// The consumers of the composed record and the constructor.
fn consumers() -> StateConsumerCensus {
    StateConsumerCensus::from_sources(&record(), &state_constructor())
}

/// Every key the composed program pushes, in key order.
fn pushed_keys() -> Vec<Key> {
    vec![
        Key::StateAsset,
        Key::StateAmount,
        Key::InternalKey,
        Key::MaturityLeadMin,
        Key::MaturityLeadMax,
        Key::CommittedOperatorKey,
    ]
}

// (a) The key vocabulary itself: fourteen, in key order, seven of them
// program-capable, while the historical record consumes six.
#[test]
fn the_key_vocabulary_is_fourteen_with_seven_program_symbols() {
    let all = Key::ALL;
    assert_eq!(all.len(), 14);

    let unique: BTreeSet<Key> = all.iter().copied().collect();
    assert_eq!(unique.len(), all.len());
    assert_eq!(unique.into_iter().collect::<Vec<_>>(), all.to_vec());

    let pushed: Vec<Key> = all
        .iter()
        .copied()
        .filter(|symbol| symbol.is_program_symbol())
        .collect();
    assert_eq!(pushed.len(), 7);
    assert_eq!(
        pushed
            .iter()
            .copied()
            .filter(|key| *key != Key::MetadataHeader)
            .collect::<Vec<_>>(),
        pushed_keys()
    );

    let composed = record();
    let mapped: BTreeSet<Key> = composed
        .consumers()
        .keys()
        .map(|symbol| Key::from_program(*symbol))
        .collect();
    assert_eq!(mapped, pushed_keys().into_iter().collect::<BTreeSet<_>>());

    let kinds: BTreeSet<Key> = state_constructor()
        .reference_declarations()
        .iter()
        .map(|declared| Key::from_reference(&declared.reference))
        .collect();
    assert_eq!(kinds.len(), 7);
    assert!(kinds.iter().all(|symbol| !symbol.is_program_symbol()));
}

// (b) Pass one: thirteen definitions, each from the layer answerable for
// it, each the type its key declares.
#[test]
fn pass_one_defines_thirteen_keys_over_five_typed_origins() {
    let target = reviewed_target();
    let census = definitions(&target);

    assert_eq!(census.len(), 13);
    assert!(!census.is_empty());

    let expected: BTreeMap<Key, Origin> = BTreeMap::from([
        (Key::StateAsset, Origin::Deployment),
        (Key::StateAmount, Origin::ArchitectureAsset),
        (Key::InternalKey, Origin::Constructor),
        (Key::MaturityLeadMin, Origin::ArchitectureBounds),
        (Key::MaturityLeadMax, Origin::ArchitectureBounds),
        (Key::CommittedOperatorKey, Origin::Deployment),
        (Key::MetadataSchema, Origin::Constructor),
        (Key::StaticSubtreeRoot, Origin::Constructor),
        (Key::LeafVersion, Origin::Constructor),
        (Key::InternalKeyPolicy, Origin::Constructor),
        (Key::BranchSide, Origin::Constructor),
        (Key::NonceBudget, Origin::Constructor),
        (Key::TargetPolicy, Origin::ReviewedTarget),
    ]);
    let observed: BTreeMap<Key, Origin> = census
        .definitions()
        .iter()
        .map(|(symbol, definition)| (*symbol, definition.origin()))
        .collect();
    assert_eq!(observed, expected);
    assert_eq!(observed.values().copied().collect::<BTreeSet<_>>().len(), 5);

    for (symbol, definition) in census.definitions() {
        assert_eq!(definition.symbol(), *symbol);
        assert_eq!(
            definition.value().symbol_type(),
            state_declared_type(*symbol)
        );
    }

    assert_eq!(
        census.from_origin(Origin::Deployment).collect::<Vec<_>>(),
        vec![Key::StateAsset, Key::CommittedOperatorKey]
    );
    assert_eq!(
        census
            .from_origin(Origin::ArchitectureAsset)
            .collect::<Vec<_>>(),
        vec![Key::StateAmount]
    );
    assert_eq!(
        census
            .from_origin(Origin::ArchitectureBounds)
            .collect::<Vec<_>>(),
        vec![Key::MaturityLeadMin, Key::MaturityLeadMax]
    );
    assert_eq!(census.from_origin(Origin::Constructor).count(), 7);
    assert_eq!(
        census
            .from_origin(Origin::ReviewedTarget)
            .collect::<Vec<_>>(),
        vec![Key::TargetPolicy]
    );

    // The declared issuance is the architecture's one unit, not a
    // deployment's number, and it arrives as the item the leaf compares.
    assert_eq!(
        census
            .definition(Key::StateAmount)
            .map(|d| d.value().clone()),
        Some(StateSymbolValue::ExplicitAmount(StackItem::signed_le64(
            &target, 1
        )))
    );
    assert_eq!(declaration().fixed_amount().get(), 1);
}

// (c) The consumer side, read off the record and the constructor: six
// pushed keys over fifteen sites, and seven keys a constructor field
// consumes without producing any push.
#[test]
fn the_consumer_census_is_six_keys_over_fifteen_sites_and_seven_fields() {
    let composed = record();
    let consumers = consumers();

    assert_eq!(consumers.consumers().len(), 13);
    assert_eq!(consumers.push_site_count(), 15);

    let counts: BTreeMap<Key, usize> = consumers
        .consumers()
        .iter()
        .map(|(symbol, sites)| (*symbol, sites.record_sites().len()))
        .collect();
    assert_eq!(
        counts,
        BTreeMap::from([
            (Key::StateAsset, 2),
            (Key::StateAmount, 2),
            (Key::InternalKey, 4),
            (Key::MaturityLeadMin, 3),
            (Key::MaturityLeadMax, 3),
            (Key::CommittedOperatorKey, 1),
            (Key::MetadataSchema, 0),
            (Key::StaticSubtreeRoot, 0),
            (Key::LeafVersion, 0),
            (Key::InternalKeyPolicy, 0),
            (Key::BranchSide, 0),
            (Key::NonceBudget, 0),
            (Key::TargetPolicy, 0),
        ])
    );

    for (symbol, sites) in consumers.consumers() {
        assert_eq!(sites.constructor_field(), !symbol.is_program_symbol());
        assert_eq!(
            sites.record_sites().is_empty(),
            !symbol.is_program_symbol(),
            "{symbol:?} has the wrong kind of consumer"
        );
    }

    // The same figures, from the record's own census: the link's keys
    // canonicalize the shared asset and amount, and no site is lost or
    // counted twice in the process.
    assert_eq!(composed.consumers().len(), 6);
    assert_eq!(
        composed
            .consumers()
            .values()
            .map(|consumer| consumer.sites.len())
            .sum::<usize>(),
        15
    );
}

// (d) Pass two over the real sources, and what each resolved value would
// be substituted with at each site it is consumed at.
#[test]
fn every_pushed_definition_carries_the_record_item_at_each_of_its_sites() {
    let target = reviewed_target();
    let composed = record();
    let resolved = resolve_state_census(&definitions(&target), &consumers())
        .expect("the demonstration census resolves");

    assert_eq!(resolved.entries().len(), 13);
    assert_eq!(resolved.push_site_count(), 15);
    assert_eq!(
        resolved.program_keys(),
        pushed_keys().into_iter().collect::<BTreeSet<_>>()
    );
    assert_eq!(resolved.constructor_keys().len(), 7);
    assert!(
        resolved
            .program_keys()
            .is_disjoint(&resolved.constructor_keys())
    );

    let instructions = composed.program().instructions();
    let mut visited = 0;

    for (symbol, entry) in resolved.entries() {
        let item = entry.definition().value().push_item(&target);

        if !symbol.is_program_symbol() {
            assert!(item.is_none(), "{symbol:?} has no site to push an item at");
            continue;
        }

        let item = item.expect("a pushed key resolves to an item");
        for &site in entry.sites().record_sites() {
            let Some(TapscriptInstruction::Push(fixture)) = instructions.get(site) else {
                panic!("consumer site {site} of {symbol:?} is not a push");
            };

            // Width holds for any deployment: the linked item occupies
            // the site the fixture occupied, so the program's shape does
            // not move when its values do.
            assert_eq!(item.len(), fixture.len());
            // Equality is these sources agreeing. The demonstration
            // deployment supplies the same asset, amount, key and window
            // the record was composed against, so a substitution here
            // would be the identity — which is why the width assertion
            // above is the one that generalizes.
            assert_eq!(&item, fixture);
            visited += 1;
        }
    }

    assert_eq!(visited, 15);
}

#[test]
fn variable_census_defines_and_consumes_the_layout_header_at_one_push_site() {
    let target = reviewed_target();
    let record = record_for_schedule(tapscript::StateWitnessSchedule::VariableMetadata);
    let definitions = collect_state_definitions(
        &target,
        &bridge(),
        &state_constructor(),
        &singleton(),
        &declaration(),
        record.schedule(),
    )
    .expect("variable definitions collect");
    let consumers = StateConsumerCensus::from_sources(&record, &state_constructor());
    let resolved =
        resolve_state_census(&definitions, &consumers).expect("variable census resolves");
    assert_eq!(definitions.len(), 14);
    assert_eq!(
        definitions
            .definitions()
            .values()
            .map(crate::StateSymbolDefinition::origin)
            .collect::<BTreeSet<_>>()
            .len(),
        6
    );
    assert_eq!(
        definitions
            .from_origin(Origin::CanonicalMetadataLayout)
            .collect::<Vec<_>>(),
        vec![Key::MetadataHeader]
    );
    assert_eq!(resolved.entries().len(), 14);
    assert_eq!(resolved.program_keys().len(), 7);
    assert_eq!(resolved.push_site_count(), 16);
    let header = &resolved.entries()[&Key::MetadataHeader];
    assert_eq!(
        header.definition().value().symbol_type(),
        StateSymbolType::MetadataHeader
    );
    assert_eq!(header.sites().record_sites().len(), 1);
    let item = header
        .definition()
        .value()
        .push_item(&target)
        .expect("header has a pushed item");
    assert_eq!(item.len(), 25);
    for &site in header.sites().record_sites() {
        assert_eq!(
            record.program().instructions().get(site),
            Some(&TapscriptInstruction::Push(item.clone()))
        );
    }
}

#[test]
fn a_header_width_disagreement_is_refused() {
    let target = reviewed_target();
    let mut layout = STATE_METADATA_LAYOUT.clone();
    layout[0].class = StateMetadataRegionClass::Constant(&[0; 20]);
    assert_eq!(
        crate::state_symbol::metadata_header_from_layout(&target, &layout, 1),
        Err(StateLinkRefusal::MetadataHeaderWidthMismatch {
            width: 24,
            variable_start: 25
        })
    );
}

#[test]
fn a_header_schema_disagreement_is_refused() {
    let target = reviewed_target();
    let mut layout = STATE_METADATA_LAYOUT.clone();
    layout[1].class = StateMetadataRegionClass::Constant(&[0, 0, 0, 2]);
    assert_eq!(
        crate::state_symbol::metadata_header_from_layout(&target, &layout, 1),
        Err(StateLinkRefusal::MetadataSchemaDisagreement {
            layout: vec![0, 0, 0, 2],
            constructor: 1
        })
    );
}

// (e) Pass one refuses a second claim rather than overwriting the first.
#[test]
fn a_second_claim_on_one_key_is_refused() {
    let target = reviewed_target();
    let mut census = StateDefinitionCensus::default();
    let value = StateSymbolValue::Asset(
        StackItem::new(&target, singleton().bytes().to_vec())
            .expect("thirty-two bytes is a literal"),
    );

    census
        .define(Key::StateAsset, value.clone(), Origin::Deployment)
        .expect("the first claim is recorded");

    assert_eq!(
        census.define(Key::StateAsset, value, Origin::Constructor),
        Err(StateLinkRefusal::DuplicateDefinition(Key::StateAsset))
    );
    assert_eq!(census.len(), 1);
}

/// The definition census with one key withheld.
fn definitions_without(
    target: &ReviewedElementsTapscriptDefinition,
    dropped: Key,
) -> StateDefinitionCensus {
    let full = definitions(target);
    let mut census = StateDefinitionCensus::default();

    for (symbol, definition) in full.definitions() {
        if *symbol == dropped {
            continue;
        }
        census
            .define(*symbol, definition.value().clone(), definition.origin())
            .expect("each key is claimed once");
    }

    census
}

// (f) Every key, one at a time: a consumer whose definition is absent is
// refused by name. Thirteen cases rather than one, because a check that
// caught only the key a test happened to pick would pass while twelve
// others went unnoticed.
#[test]
fn a_consumed_key_with_no_definition_is_refused() {
    let target = reviewed_target();
    let consumers = consumers();

    for dropped in Key::ALL
        .into_iter()
        .filter(|key| *key != Key::MetadataHeader)
    {
        let census = definitions_without(&target, dropped);
        assert_eq!(census.len(), 12);
        assert_eq!(
            resolve_state_census(&census, &consumers),
            Err(StateLinkRefusal::MissingDefinition(dropped))
        );
    }
}

// (g) The closure statement, from the other side: a definition nothing
// consumes is refused, one key at a time. An enum variant does not
// disappear when a push site does, so this is what a record that stopped
// consuming a key would run into.
#[test]
fn a_definition_no_consumer_reads_is_refused() {
    let target = reviewed_target();
    let census = definitions(&target);
    let full = consumers();

    for dropped in Key::ALL
        .into_iter()
        .filter(|key| *key != Key::MetadataHeader)
    {
        let mut map = full.consumers().clone();
        map.remove(&dropped);

        assert_eq!(
            resolve_state_census(&census, &StateConsumerCensus::new(map)),
            Err(StateLinkRefusal::UnusedDefinition(dropped))
        );
    }
}

// (h) Width is not kind: a lead bound offered for the asset is the right
// shape of value and the wrong meaning, and pass two says so with both
// types.
#[test]
fn a_definition_of_the_wrong_kind_is_refused() {
    let target = reviewed_target();
    let mut census = StateDefinitionCensus::default();

    for (symbol, definition) in definitions(&target).definitions() {
        let value = if *symbol == Key::StateAsset {
            StateSymbolValue::LeadBound(Cycle::new(2))
        } else {
            definition.value().clone()
        };
        census
            .define(*symbol, value, definition.origin())
            .expect("each key is claimed once");
    }

    assert_eq!(census.len(), 13);
    assert_eq!(
        resolve_state_census(&census, &consumers()),
        Err(StateLinkRefusal::IncompatibleType {
            symbol: Key::StateAsset,
            declared: StateSymbolType::Asset,
            offered: StateSymbolType::LeadBound,
        })
    );
}

// (i) The declaration has to be the planned object's own. The asset here
// is a declared singleton in its own right — a fixed amount of one and no
// reissuance — so what the refusal catches is the identity, not the
// shape.
#[test]
fn a_declaration_for_another_asset_is_refused() {
    let target = reviewed_target();
    let other = ARCHITECTURE
        .asset(AssetId::Pace)
        .expect("the pace authority asset is declared");
    let declaration =
        StateSingletonDeclaration::from_architecture_asset(other).expect("it declares a singleton");

    assert_eq!(*bridge().plan().state().asset(), AssetId::Pid);
    assert_eq!(declaration.asset(), AssetId::Pace);
    assert_eq!(
        collect_state_definitions(
            &target,
            &bridge(),
            &state_constructor(),
            &singleton(),
            &declaration,
            tapscript::StateWitnessSchedule::WholeMetadata,
        ),
        Err(StateLinkRefusal::SingletonDeclarationMismatch)
    );
}

// (j) Why no test reaches the revision disagreement, stated as the two
// facts that make it unreachable rather than as a claim: exactly one
// reviewed contract revision can be constructed, and both operands of
// the comparison are read from it.
#[test]
fn the_revision_check_has_one_reviewed_revision_to_agree_on() {
    let target = reviewed_target();
    let supported = [TargetContractVersion::V2];
    assert_eq!(TargetContractVersion::SUPPORTED, &supported);
    assert_eq!(bridge().revision(), target.definition().version());

    let declared: Vec<TargetContractVersion> = state_constructor()
        .reference_declarations()
        .into_iter()
        .filter_map(|declared| match declared.reference {
            StateConstructorReference::TargetPolicy(revision) => Some(revision),
            _ => None,
        })
        .collect();
    assert_eq!(declared, vec![target.definition().version()]);
}

/// Which test reaches one refusal, or why nothing can.
///
/// An exhaustive match with no wildcard, so a variant added to the closed
/// root does not compile until it is accounted for here.
fn reachability(refusal: &StateLinkRefusal) -> &'static str {
    match refusal {
        StateLinkRefusal::DuplicateDefinition(_) => "a_second_claim_on_one_key_is_refused",
        StateLinkRefusal::MissingDefinition(_) => "a_consumed_key_with_no_definition_is_refused",
        StateLinkRefusal::UnusedDefinition(_) => "a_definition_no_consumer_reads_is_refused",
        StateLinkRefusal::IncompatibleType { .. } => "a_definition_of_the_wrong_kind_is_refused",
        StateLinkRefusal::MetadataHeaderWidthMismatch { .. } => {
            "a_header_width_disagreement_is_refused"
        }
        StateLinkRefusal::MetadataSchemaDisagreement { .. } => {
            "a_header_schema_disagreement_is_refused"
        }
        StateLinkRefusal::SingletonDeclarationMismatch => {
            "a_declaration_for_another_asset_is_refused"
        }
        StateLinkRefusal::TargetRevisionDisagreement { .. } => {
            "unreachable: one reviewed revision is constructible and both operands read from it"
        }
        StateLinkRefusal::InvalidDefinitionItem { .. } => {
            "unreachable: every value is eight or thirty-two bytes, under the reviewed bound"
        }
        StateLinkRefusal::FrozenGraph(_) => {
            "unreachable: the constructor states seven references against a bound of sixty-four"
        }
        StateLinkRefusal::GraphReferenceLimitExceeded { .. } => {
            "the_shared_reference_bound_admits_sixty_four_nodes_and_no_more"
        }
        StateLinkRefusal::LiteralStaticRootBeneathItself { .. } => {
            "a_link_time_static_root_under_its_own_program_is_refused"
        }
        StateLinkRefusal::ConstructorProjectionMismatch { .. } => {
            "a_constructor_kind_missing_from_the_graph_is_a_projection_mismatch"
        }
        StateLinkRefusal::UnvalidatedCut { .. } => {
            "a_witnessed_root_without_the_root_witness_is_not_a_cut"
        }
        StateLinkRefusal::ResidualCycle { .. } => "one_cut_leaves_the_other_edge_disjoint_cycle",
        // The relocation and resource halves account for their own
        // variants beside the tests that reach them. Each is named here
        // rather than swept up by a wildcard, so a variant added to the
        // closed root still fails to compile until somebody accounts
        // for it.
        StateLinkRefusal::UnresolvedRelocation(_)
        | StateLinkRefusal::SiteIsNotAPush { .. }
        | StateLinkRefusal::SiteOutsideEveryComponent { .. }
        | StateLinkRefusal::LiteralStaticRootRelocation
        | StateLinkRefusal::RelocationCensusDisagreement { .. }
        | StateLinkRefusal::RelocationNotApplied { .. }
        | StateLinkRefusal::UntrackedProgramMutation { .. }
        | StateLinkRefusal::RoundTripMismatch
        | StateLinkRefusal::LinkedProgramDoesNotSchedule { .. }
        | StateLinkRefusal::AbstractExecutionMoved
        | StateLinkRefusal::ResourceDeltaOverflow { .. } => {
            crate::tests::state_relocate_tests::relocation_reachability(refusal)
        }
        StateLinkRefusal::ResourceTotalOverflow { .. }
        | StateLinkRefusal::LinkedProgramFailsTheFinalStackRule { .. }
        | StateLinkRefusal::ResourceProjectionDisagreement { .. } => {
            crate::tests::state_resource_tests::resource_reachability(refusal)
        }
        StateLinkRefusal::EmittedProjection(_)
        | StateLinkRefusal::MissingCompilerRelation { .. }
        | StateLinkRefusal::MissingEmittedRow { .. }
        | StateLinkRefusal::ExtraEmittedRow { .. }
        | StateLinkRefusal::DuplicateCarrierRow { .. }
        | StateLinkRefusal::DischargeBoundaryDisagreement { .. }
        | StateLinkRefusal::ExternalRequirementDropped { .. }
        | StateLinkRefusal::VacuityDisagreement { .. }
        | StateLinkRefusal::DeploymentFactUnrecorded { .. }
        | StateLinkRefusal::MissingComponentRange { .. }
        | StateLinkRefusal::ComponentRangeOutsideLeaf { .. }
        | StateLinkRefusal::CarrierLeafUncommitted { .. }
        | StateLinkRefusal::ComponentRangeMoved { .. }
        | StateLinkRefusal::SelectedAlternativeUnmatched { .. }
        | StateLinkRefusal::CensusNotTotal { .. }
        | StateLinkRefusal::RepresentationDisagreement { .. } => {
            crate::tests::state_carrier_tests::carrier_reachability(refusal)
        }
        StateLinkRefusal::SuppliedConstructorCommitsAnotherProgram
        | StateLinkRefusal::ConstructorApplication(_)
        | StateLinkRefusal::AppliedReferenceDisagreement { .. }
        | StateLinkRefusal::CensusMovedUnderApplication { .. }
        | StateLinkRefusal::StaticSubtreeDiscontinuity { .. }
        | StateLinkRefusal::InstanceAlreadyRetained { .. }
        | StateLinkRefusal::ConstructorPolicyIncomplete { .. } => {
            crate::tests::state_bundle_tests::bundle_reachability(refusal)
        }
    }
}

// (k) The closed root, enumerated.
#[test]
fn the_closed_refusal_root_is_covered_by_test_or_by_reason() {
    let target = reviewed_target();
    // A real refusal from the item constructor, because the adapter's
    // error root admits no variant built from outside it — and because a
    // fabricated cause would say nothing about the bound that makes the
    // item refusal unreachable for a thirty-two-byte value.
    let cause = StackItem::new(&target, vec![0; 1_000_000])
        .expect_err("a million bytes is not a literal the reviewed contract admits");

    let refusals = [
        StateLinkRefusal::DuplicateDefinition(Key::StateAsset),
        StateLinkRefusal::MissingDefinition(Key::StateAmount),
        StateLinkRefusal::UnusedDefinition(Key::NonceBudget),
        StateLinkRefusal::IncompatibleType {
            symbol: Key::StateAsset,
            declared: StateSymbolType::Asset,
            offered: StateSymbolType::LeadBound,
        },
        StateLinkRefusal::MetadataHeaderWidthMismatch {
            width: 24,
            variable_start: 25,
        },
        StateLinkRefusal::MetadataSchemaDisagreement {
            layout: vec![0, 0, 0, 2],
            constructor: 1,
        },
        StateLinkRefusal::TargetRevisionDisagreement {
            constructor: TargetContractVersion::V1,
            reviewed: TargetContractVersion::V2,
        },
        StateLinkRefusal::SingletonDeclarationMismatch,
        StateLinkRefusal::InvalidDefinitionItem {
            symbol: Key::InternalKey,
            cause,
        },
        StateLinkRefusal::FrozenGraph(StateReferenceGraphRefusal::ResourceBoundExceeded {
            limit: STATE_REFERENCE_LIMIT,
        }),
        StateLinkRefusal::GraphReferenceLimitExceeded {
            limit: STATE_REFERENCE_LIMIT,
        },
        StateLinkRefusal::LiteralStaticRootBeneathItself {
            program: StateLeafRole::Announcement,
        },
        StateLinkRefusal::ConstructorProjectionMismatch {
            frozen: BTreeSet::from([Key::TargetPolicy]),
            graph: BTreeSet::new(),
        },
        StateLinkRefusal::UnvalidatedCut {
            referrer: StateGraphNode::Program(StateLeafRole::Announcement),
            referent: StateGraphNode::Definition(Key::StaticSubtreeRoot),
            binding: StateBindingTime::WitnessedThenAuthenticated,
            missing_witnesses: vec![StateProgramWitness::StaticSubtreeRoot],
            missing_components: BTreeSet::new(),
        },
        // The residual cycle is the refusal the linker built, because a
        // component has no constructor outside the analysis and an
        // invented one would stand for nothing.
        residual_cycle_refusal(),
    ];

    let accounts: BTreeSet<&str> = refusals.iter().map(reachability).collect();
    assert_eq!(refusals.len(), 15);
    assert_eq!(accounts.len(), 15);
    assert_eq!(
        refusals
            .iter()
            .filter(|refusal| reachability(refusal).starts_with("unreachable"))
            .count(),
        3
    );
}
