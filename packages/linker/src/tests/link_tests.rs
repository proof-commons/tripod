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
//! # The headline result of this wave is a refusal
//!
//! With no self-commitment strategy stated — the honest default — the
//! demonstration bundle does not link. The ASH constructor's witness
//! program is the taproot output committing to the taptree over the
//! twelve leaves, and those leaves push that program as a link-time
//! literal, so the symbol's value is a function of itself. §14.4 calls
//! that an impossible static fixed point, prohibits searching for it by
//! repeated hashing, and requires an explicit authenticated strategy.
//! Both the refusal and the sound alternative's refusal are asserted
//! below, because both are findings rather than accidents.

use std::collections::BTreeSet;

use tapscript::{BundleSymbol, LeafRole, ProgramRole, ResourceModel};
use target_elements::ResourceDimension;

use crate::tests::{deployment, relocatable_bundle, reviewed_target};
use crate::{
    CandidateLinkedBundle, LinkObligation, LinkRefusal, LinkedArtifactStatus,
    SelfCommitmentStrategy, link_candidate,
};

/// The demonstration linked bundle, under the one strategy that links.
fn linked() -> CandidateLinkedBundle {
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
fn the_sound_strategy_is_refused_because_the_leaves_still_push_a_literal() {
    // The strategy that would actually resolve the cycle is for the
    // referring programs to read the identity from the target at spend
    // time instead of carrying a literal. Declaring it does not make it
    // true, and the linker checks rather than believes: the emitted
    // leaves do push the literal, so the declaration is contradicted
    // and named against the leaf that contradicts it.
    //
    // That is a finding about the backend, not about this linker. Until
    // the recognition fragments introspect the spending input's own
    // program, no link of this bundle can be both sound and complete.
    let target = reviewed_target();
    let refusal = link_candidate(
        &target,
        &relocatable_bundle(),
        &deployment(&target, SelfCommitmentStrategy::IdentityIntrospection),
    )
    .expect_err("the emitted leaves contradict an introspection strategy");

    match refusal {
        LinkRefusal::CycleStrategyContradictedByRelocation { symbol, .. } => {
            assert_eq!(symbol, BundleSymbol::AshConstructorProgram);
        }
        other => panic!("expected a contradicted strategy, got {other:?}"),
    }
}

#[test]
fn the_authenticated_link_carries_its_undischarged_equality_structurally() {
    // The one strategy that links cuts the cycle by authority rather
    // than by computation, and the whole content of that is an
    // obligation nothing in this wave discharges. It is a field, not a
    // sentence: a bundle linked this way cannot be read as one whose
    // commitment was checked, because the obligation set is non-empty
    // by type and this member of it is present by construction.
    let bundle = linked();
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
    assert_eq!(bundle.unresolved_target_evidence().len(), 12);
}

#[test]
fn every_symbol_the_bundle_declared_is_defined_exactly_once() {
    // The definition census against the bundle's own symbol table: the
    // same keys, no more and no fewer. Eight symbols the deployment
    // settles and fifteen the bundle does, which is the eleven scalars
    // plus the twelve leaf scripts less the eight — written out so a
    // symbol that quietly changed sides fails here.
    let bundle = linked();
    let relocatable = relocatable_bundle();

    assert_eq!(bundle.definitions().len(), 23);
    assert_eq!(relocatable.symbols().len(), 23);
    let defined: BTreeSet<BundleSymbol> =
        bundle.definitions().definitions().keys().copied().collect();
    let declared: BTreeSet<BundleSymbol> = relocatable.symbols().keys().copied().collect();
    assert_eq!(defined, declared);

    let from_deployment: BTreeSet<BundleSymbol> = bundle
        .definitions()
        .from_origin(crate::DefinitionOrigin::DeploymentParameters)
        .collect();
    assert_eq!(
        from_deployment,
        BTreeSet::from([
            BundleSymbol::ClosedAsset,
            BundleSymbol::ReserveAsset,
            BundleSymbol::AshConstructorProgram,
            BundleSymbol::AshConstructorProgramVersion,
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

        let ash = pushes(*leaf, BundleSymbol::AshConstructorProgram);
        let change = pushes(*leaf, BundleSymbol::SponsorChangeProgram);
        assert_eq!(
            after - before,
            12 * (change - ash),
            "leaf {leaf:?} moved by an amount the resolved widths do not explain",
        );
    }

    // And the totals, written out, so a leaf that changed without its
    // neighbours noticing fails here too.
    assert_eq!(relocatable.total_script_bytes(), 5169);
    assert_eq!(bundle.total_script_bytes(), 4737);
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
    // the constructor node, and every leaf-script symbol is in it —
    // which is precisely the statement that the leaves commit to the
    // program that commits to the leaves.
    let bundle = linked();
    let graph = bundle.reference_graph();

    assert_eq!(graph.cycles().count(), 1);
    let cycle = graph.cycles().next().expect("one cycle");
    assert!(cycle.members().contains(&crate::ReferenceNode::Symbol(
        BundleSymbol::AshConstructorProgram
    )));
    assert!(cycle.members().contains(&crate::ReferenceNode::Constructor));

    for leaf in bundle.programs().keys() {
        assert!(
            cycle
                .members()
                .contains(&crate::ReferenceNode::Symbol(crate::leaf_symbol(*leaf))),
            "leaf {leaf:?} is outside the component it is committed by",
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
