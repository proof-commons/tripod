//! The demonstration link, end to end (§14).
//!
//! # This is a fixture link, and it says so
//!
//! The symbol values resolved here are public, meaningless byte
//! strings — test material in the sense
//! `(´[ADR015-rule:security:test-material]´)` fixes. A production
//! deployment's asset identifiers, programs, and internal key are not
//! this linker's to invent (§1.10), so what the tests below establish
//! is that the *mechanism* resolves, substitutes, commits, and closes
//! correctly, not that any particular deployment exists.
//!
//! # The self-commitment, and how it is resolved
//!
//! The ASH constructor's witness program is the taproot output
//! committing to the taptree over the twelve leaves, and the leaves
//! depend on that program: the coordinator's recognition fragments
//! compare other positions against it. The dependency is a cycle
//! whichever way it is discharged, and §14.4 requires an explicit
//! authenticated strategy for it rather than a silent choice.
//!
//! The coordinator leaves read the program from the input they are
//! spending instead of carrying a literal for it, which is what makes
//! [`SelfCommitmentStrategy::IdentityIntrospection`] a resolution
//! rather than a claim: there is no literal to be wrong, and no value
//! for a deployment to supply. That strategy links, and the link is
//! asserted below. With no strategy stated the same cycle is still
//! there and still unclassified, so the honest default still refuses —
//! that refusal is asserted too, because a resolution nobody could
//! have skipped is not much of a resolution.

use std::collections::BTreeSet;
use std::num::NonZeroUsize;

use tapscript::{
    BundleSymbol, FieldSide, LeafRole, ProgramRole, Relocation, RelocationEncoding, RelocationSite,
    ResourceModel, StackItem, SymbolWidth, TargetRole,
};
use target_elements::{EncodingClass, ResourceDimension};

use crate::tests::{deployment, relocatable_bundle, reviewed_target};
use crate::{
    CandidateLinkedBundle, LinkObligation, LinkRefusal, LinkedArtifactStatus,
    SelfCommitmentStrategy, link_candidate,
};

/// The demonstration linked bundle, under the sound strategy.
fn linked() -> CandidateLinkedBundle {
    let target = reviewed_target();
    link_candidate(
        &target,
        &relocatable_bundle(),
        &deployment(&target, SelfCommitmentStrategy::IdentityIntrospection),
    )
    .expect("the demonstration link completes under identity introspection")
}

/// The same link under the strategy that cuts the cycle by authority.
///
/// Kept as a second fixture rather than as the main one: it still
/// links, and what it carries that the sound link does not is the
/// undischarged equality obligation.
fn authenticated() -> CandidateLinkedBundle {
    let target = reviewed_target();
    link_candidate(
        &target,
        &relocatable_bundle(),
        &deployment(
            &target,
            SelfCommitmentStrategy::ExternallyAuthenticatedCommitment,
        ),
    )
    .expect("the demonstration link completes under an authenticated commitment")
}

#[test]
fn an_unstated_strategy_refuses_the_self_referential_constructor() {
    // The default, and the wave's headline finding. Not "the linker is
    // incomplete": the linker reached the answer, and the answer is
    // that the emitted design asks for a program whose own bytes appear
    // inside the tree it commits to. The refusal names the component
    // and the symbol, so it is actionable rather than a shrug.
    let target = reviewed_target();
    let refusal = link_candidate(
        &target,
        &relocatable_bundle(),
        &deployment(&target, SelfCommitmentStrategy::NotStated),
    )
    .expect_err("an unstated strategy cannot resolve the cycle");

    match refusal {
        LinkRefusal::ImpossibleStaticFixedPoint { symbol, .. } => {
            assert_eq!(symbol, BundleSymbol::AshConstructorProgram);
        }
        other => panic!("expected an impossible static fixed point, got {other:?}"),
    }
}

#[test]
fn the_sound_strategy_links_and_owes_no_equality() {
    // The resolution. The referring programs read the identity from the
    // input they are spending, so there is no literal to contradict the
    // declaration and no supplied value whose equality with the tree
    // would have to be taken on trust. The link completes, and the
    // obligation the authenticated strategy carries is absent here —
    // which is the difference between cutting the cycle and cutting it
    // soundly.
    let bundle = linked();

    assert_eq!(
        bundle.self_commitment(),
        SelfCommitmentStrategy::IdentityIntrospection
    );
    let obligations = bundle.outstanding_obligations();
    assert_eq!(obligations.count().get(), 2);
    assert!(!obligations.holds(LinkObligation::SelfCommitmentEqualityUndischarged));
    assert!(obligations.holds(LinkObligation::TaprootOutputKeyUndischarged));
    assert!(obligations.holds(LinkObligation::InternalKeyUnspendabilityUnverified));
}

#[test]
fn a_literal_reaching_a_leaf_still_contradicts_the_introspection_strategy() {
    // The check that made the old design's refusal, kept live against a
    // relocation set stated here rather than emitted. The bundles this
    // crate links no longer place the ASH program into a program at
    // all, so without this the refusal would be unreachable code that
    // only reasoning defends.
    let target = reviewed_target();
    let placeholder = StackItem::new(&target, vec![0x00; 32]).expect("a literal within the bound");
    let leaf = LeafRole::Member { ash_inputs: 2 };
    let relocation = Relocation::byte_patch(
        BundleSymbol::AshConstructorProgram,
        TargetRole::ProgramComparand {
            side: FieldSide::Input,
        },
        RelocationSite::ProgramInstructions {
            leaf,
            indices: BTreeSet::from([7]),
        },
        SymbolWidth::Fixed { bytes: 32 },
        RelocationEncoding::EncodedPayload {
            class: EncodingClass::WitnessProgram,
        },
        NonZeroUsize::MIN,
        placeholder,
    )
    .expect("a fixed width with a matching placeholder is admissible");

    assert_eq!(
        crate::bundle::no_literal_reaches_a_leaf(std::iter::once(&relocation)),
        Err(LinkRefusal::CycleStrategyContradictedByRelocation {
            symbol: BundleSymbol::AshConstructorProgram,
            leaf,
        }),
    );

    // And a relocation that binds the constructor rather than a program
    // is not a contradiction, so the check is discriminating.
    let binding = Relocation::byte_patch(
        BundleSymbol::AshConstructorProgram,
        TargetRole::ProgramComparand {
            side: FieldSide::Input,
        },
        RelocationSite::ConstructorBinding,
        SymbolWidth::Fixed { bytes: 32 },
        RelocationEncoding::EncodedPayload {
            class: EncodingClass::WitnessProgram,
        },
        NonZeroUsize::MIN,
        StackItem::new(&target, vec![0x00; 32]).expect("a literal within the bound"),
    )
    .expect("a fixed width with a matching placeholder is admissible");
    assert_eq!(
        crate::bundle::no_literal_reaches_a_leaf(std::iter::once(&binding)),
        Ok(()),
    );
}

#[test]
fn the_authenticated_link_carries_its_undischarged_equality_structurally() {
    // The one strategy that links cuts the cycle by authority rather
    // than by computation, and the whole content of that is an
    // obligation nothing in this wave discharges. It is a field, not a
    // sentence: a bundle linked this way cannot be read as one whose
    // commitment was checked, because the obligation set is non-empty
    // by type and this member of it is present by construction.
    let bundle = authenticated();
    let obligations = bundle.outstanding_obligations();

    assert_eq!(obligations.count().get(), 3);
    assert!(obligations.holds(LinkObligation::SelfCommitmentEqualityUndischarged));
    assert!(obligations.holds(LinkObligation::TaprootOutputKeyUndischarged));
    assert!(obligations.holds(LinkObligation::InternalKeyUnspendabilityUnverified));
    assert_eq!(
        bundle.self_commitment(),
        SelfCommitmentStrategy::ExternallyAuthenticatedCommitment
    );
}

#[test]
fn the_linked_bundle_is_a_candidate_and_has_no_way_to_say_otherwise() {
    // §1.9 and §1.10 together: the status is read-only and always
    // prototype, the backend's outstanding lifecycle survives the link,
    // and the target evidence the correctness rests on is still
    // unresolved. A bundle that had linked its way to a promotion would
    // fail all three.
    let bundle = linked();

    assert_eq!(bundle.status(), LinkedArtifactStatus::Prototype);
    assert_eq!(bundle.outstanding_lifecycle().len(), 1);
    assert_eq!(bundle.unresolved_target_evidence().len(), 13);
}

#[test]
fn every_symbol_the_bundle_declared_is_defined_exactly_once() {
    // The definition census against the bundle's own symbol table: the
    // same keys but one, and the one exception is the point. Six
    // symbols the deployment settles and fifteen the bundle does, out
    // of a table of twenty-two — the ASH constructor's program is
    // declared and defined by nobody, because no layer can settle it.
    // Written out so a symbol that quietly changed sides fails here.
    let bundle = linked();
    let relocatable = relocatable_bundle();

    assert_eq!(relocatable.symbols().len(), 22);
    assert_eq!(bundle.definitions().len(), 21);
    let defined: BTreeSet<BundleSymbol> =
        bundle.definitions().definitions().keys().copied().collect();
    let declared: BTreeSet<BundleSymbol> = relocatable.symbols().keys().copied().collect();
    assert_eq!(
        declared.difference(&defined).copied().collect::<Vec<_>>(),
        vec![BundleSymbol::AshConstructorProgram],
    );

    let from_deployment: BTreeSet<BundleSymbol> = bundle
        .definitions()
        .from_origin(crate::DefinitionOrigin::DeploymentParameters)
        .collect();
    assert_eq!(
        from_deployment,
        BTreeSet::from([
            BundleSymbol::ClosedAsset,
            BundleSymbol::ReserveAsset,
            BundleSymbol::SponsorChangeProgram,
            BundleSymbol::SponsorChangeProgramVersion,
            BundleSymbol::TargetFeeRoleProgramDigest,
            BundleSymbol::UnspendableInternalKey,
        ])
    );
}

#[test]
fn the_committed_tree_settles_the_control_path_depth_the_backend_owed() {
    // The relocatable bundle listed `ControlPathDepth` as owed to the
    // linker. After linking it is a figure rather than an obligation,
    // and the figure is the one the closed form gives for twelve
    // equal-weight leaves: eight at depth four and four at depth three,
    // for a total weighted depth of forty-four and a deepest path of
    // four.
    let bundle = linked();

    assert_eq!(bundle.control_path_depth(), 4);
    assert_eq!(bundle.taptree().cost(), 44);
    assert!(
        !bundle
            .outstanding_dimensions()
            .contains_key(&ResourceDimension::ControlPathDepth)
    );
    assert_eq!(bundle.outstanding_dimensions().len(), 5);

    let deep = bundle
        .taptree()
        .recipes()
        .values()
        .filter(|recipe| recipe.depth() == 4)
        .count();
    let shallow = bundle
        .taptree()
        .recipes()
        .values()
        .filter(|recipe| recipe.depth() == 3)
        .count();
    assert_eq!((deep, shallow), (8, 4));
}

#[test]
fn the_linked_script_bytes_are_the_pre_link_ones_moved_by_the_resolved_widths() {
    // The sharpest check in this file, and the reason the fixture
    // resolutions differ in width from the placeholders. Every byte of
    // the difference is accounted for arithmetically rather than
    // observed: the ASH program resolves from thirty-two bytes to
    // twenty, so every push of it loses twelve bytes; the sponsor
    // change program resolves from twenty to thirty-two, so every push
    // of it gains twelve. A leaf's linked size is its pre-link size
    // plus twelve times (change pushes minus ASH pushes), and nothing
    // else moved.
    let relocatable = relocatable_bundle();
    let bundle = linked();

    let pushes = |leaf: LeafRole, symbol: BundleSymbol| -> i64 {
        relocatable
            .relocations_for(symbol)
            .filter_map(|relocation| match relocation.site() {
                tapscript::RelocationSite::ProgramInstructions {
                    leaf: site_leaf,
                    indices,
                } if *site_leaf == leaf => Some(i64::try_from(indices.len()).unwrap_or_default()),
                _ => None,
            })
            .sum()
    };

    for (leaf, pre) in relocatable.constructor().leaves() {
        let before = i64::try_from(
            pre.resources()
                .charged(ResourceDimension::ScriptBytes)
                .expect("the pre-link leaf charges script bytes"),
        )
        .expect("a small figure fits");
        let after = i64::try_from(
            bundle
                .program(*leaf)
                .expect("every leaf survives the link")
                .charged(ResourceDimension::ScriptBytes)
                .expect("the linked leaf charges script bytes"),
        )
        .expect("a small figure fits");

        // One symbol's resolution changes width: the sponsor-change
        // program goes from a twenty byte placeholder to a thirty-two
        // byte resolution. Nothing else moves, and the ASH constructor
        // program moves nothing at all because no leaf carries it.
        assert_eq!(pushes(*leaf, BundleSymbol::AshConstructorProgram), 0);
        let change = pushes(*leaf, BundleSymbol::SponsorChangeProgram);
        assert_eq!(
            after - before,
            12 * change,
            "leaf {leaf:?} moved by an amount the resolved widths do not explain",
        );
    }

    // And the totals, written out, so a leaf that changed without its
    // neighbours noticing fails here too.
    assert_eq!(relocatable.total_script_bytes(), 3939);
    assert_eq!(bundle.total_script_bytes(), 3975);
}

#[test]
fn the_resource_formulas_are_refitted_and_predict_every_linked_measurement() {
    // §13.3's affine model over the linked figures, not the pre-link
    // ones. The coefficients are checked by prediction at every shape
    // rather than by being written down: a model that missed one shape
    // would be the partial fit §1.11 refuses.
    let bundle = linked();
    let formulas = bundle
        .formulas()
        .get(&ProgramRole::Coordinator)
        .expect("the coordinator role has formulas")
        .get(&ResourceDimension::ScriptBytes)
        .expect("script bytes are modelled");

    assert_eq!(formulas.measurements().len(), 9);
    for (shape, measured) in formulas.measurements() {
        let predicted = tapscript::predict_shape_model(formulas.model(), *shape)
            .expect("an affine model predicts");
        assert_eq!(
            u64::try_from(predicted).expect("a positive prediction"),
            *measured,
            "the refitted model misses {shape:?}",
        );
    }

    // The pre-link model described programs that no longer exist, so
    // the two must differ. Equal coefficients would mean the refit did
    // not happen.
    let pre = relocatable_bundle();
    let before = pre
        .formulas()
        .get(&ProgramRole::Coordinator)
        .expect("the pre-link coordinator role has formulas")
        .get(&ResourceDimension::ScriptBytes)
        .expect("script bytes are modelled")
        .model();
    assert_ne!(before, formulas.model());
    assert!(matches!(formulas.model(), ResourceModel::Affine { .. }));
}

#[test]
fn the_reference_graph_holds_exactly_one_cycle_and_it_is_the_self_commitment() {
    // §14.4 asks the linker to find components, not to assume there are
    // none. There is one, it contains the ASH constructor program and
    // the constructor node, and every *coordinator* leaf is in it —
    // which is precisely the statement that those leaves depend on the
    // program that commits to them.
    //
    // Member leaves are outside it, and that is a result rather than a
    // gap. A member leaf compares nothing against the constructor: it
    // has only its own input in view, so the only test it could make
    // would compare that input with itself. Nothing points from a
    // member back to the program, so no loop runs through one.
    let bundle = linked();
    let graph = bundle.reference_graph();

    assert_eq!(graph.cycles().count(), 1);
    let cycle = graph.cycles().next().expect("one cycle");
    assert!(cycle.members().contains(&crate::ReferenceNode::Symbol(
        BundleSymbol::AshConstructorProgram
    )));
    assert!(cycle.members().contains(&crate::ReferenceNode::Constructor));

    for leaf in bundle.programs().keys() {
        let inside = cycle
            .members()
            .contains(&crate::ReferenceNode::Symbol(crate::leaf_symbol(*leaf)));
        assert_eq!(
            inside,
            matches!(leaf, LeafRole::Coordinator { .. }),
            "leaf {leaf:?} is on the wrong side of the component",
        );
    }
}

#[test]
fn the_linker_authors_no_digest_of_its_own() {
    // §1.10 admits a digest only once a consumer of one exists, and
    // forbids a candidate reserving a field for a future one. The check
    // is on the parts this crate authors — the constructor, the tree,
    // the definition census, the obligations, the graph — rather than
    // on the whole value, because the whole value embeds the compiler's
    // validated plan and that plan's vocabulary is not this crate's to
    // police.
    let bundle = linked();
    let authored = format!(
        "{:?}{:?}{:?}{:?}{:?}",
        bundle.constructor(),
        bundle.taptree(),
        bundle.definitions(),
        bundle.outstanding_obligations(),
        bundle.reference_graph(),
    );

    // "Digest" is deliberately absent from the list. The fee role's
    // program digest is a symbol the *target* defines and the
    // deployment resolves — an inbound value, not an identity this
    // crate minted — so its name appearing here is the census working.
    for forbidden in ["hash", "Hash", "merkle", "Merkle", "BundleIdentity"] {
        assert!(
            !authored.contains(forbidden),
            "a linker-authored value renders {forbidden}",
        );
    }
}
