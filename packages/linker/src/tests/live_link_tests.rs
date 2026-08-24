//! The candidate linked live-transfer bundle, end to end (§11).
//!
//! # The bundles under test are real ones
//!
//! Every fixture reaches its relocatable bundle the way an external
//! consumer would: derive the realization, bind the compiler input, plan
//! the live-transfer target operation, derive a constructor per
//! representation, and emit through the backend's own public entry point.
//! Nothing here hand-assembles a bundle, because a hand-assembled bundle
//! would let a link succeed against an artifact no backend produced.
//!
//! # The symbol values are fixtures and say so
//!
//! The resolutions below are distinguishable, meaningless byte strings —
//! test material in the sense `(´[ADR015-rule:security:test-material]´)`
//! fixes: public, carrying no secret, and standing for no real object.
//! The owner keys likewise: they are public metadata at the reviewed
//! encoding's exact width, and no private key exists anywhere in this
//! file or in anything it calls.

use std::collections::BTreeSet;

use tapscript::upstream::LiveTransferRepresentationPlan;
use tapscript::{
    LiveProgramRole, LiveTransferLeafRole, OwnerProfileDisposition, RecognitionResidual,
    ResourceObligation,
};
use target_elements::ResourceDimension;

use super::{live_bundles, live_deployment, live_owner, reviewed_target, single_live_bundle};
use crate::live_bundle::{LinkedLiveConstructor, LiveLinkObligation, link_live_candidate};
use crate::live_symbol::{
    LiveDefinitionOrigin, LiveLinkRole, LiveLinkSymbol, OwnerParameter, link_role_defects,
};
use crate::taptree::ExactOptimumRoute;
use crate::{LinkRefusal, LinkedArtifactStatus};

// --- The link completes -----------------------------------------------

#[test]
fn the_demonstration_link_completes_over_both_representations() {
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the demonstration live link completes");

    assert_eq!(
        linked.representation_plans(),
        &BTreeSet::from([
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ]),
    );
    assert_eq!(linked.constructors().len(), 2);
    assert_eq!(linked.status(), LinkedArtifactStatus::Prototype);
}

#[test]
fn every_constructor_is_owner_parameterized_and_keyed_by_representation() {
    // §7.6 and §11.2 together: one constructor symbol per (owner,
    // representation). The two constructors below share an owner and
    // differ in representation, so the key really is a pair rather than
    // an owner with a representation attached as a note.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");
    let owner = OwnerParameter::new(live_owner());

    for representation in [
        LiveTransferRepresentationPlan::Explicit,
        LiveTransferRepresentationPlan::PrivateCommitted,
    ] {
        let constructor = linked
            .constructor(&owner, representation)
            .expect("each representation has one constructor for this owner");
        assert_eq!(constructor.owner(), &owner);
        assert_eq!(constructor.representation(), representation);
        assert_eq!(constructor.programs().len(), 29);
        for leaf in constructor.programs().keys() {
            assert_eq!(leaf.representation(), representation);
        }
    }

    // The two constructors are different values, which is what makes the
    // owner-and-representation key load-bearing rather than decorative.
    assert_ne!(
        linked
            .constructor(&owner, LiveTransferRepresentationPlan::Explicit)
            .expect("explicit")
            .programs(),
        linked
            .constructor(&owner, LiveTransferRepresentationPlan::PrivateCommitted)
            .expect("private")
            .programs(),
    );
}

#[test]
fn two_owners_link_to_two_constructors_under_one_representation() {
    // The parameterization's whole point. Two constructors differing
    // only in the committed owner produce two linked constructors, two
    // distinct program sets, and two distinct symbol keys — and the
    // second does not overwrite the first, which a name-mangled key
    // would have risked and an ordinal key would have guaranteed.
    let target = reviewed_target();
    let first = single_live_bundle(LiveTransferRepresentationPlan::Explicit, &[0x11; 32]);
    let second = single_live_bundle(LiveTransferRepresentationPlan::Explicit, &[0x33; 32]);
    let linked = link_live_candidate(&target, &[first, second], &live_deployment(&target))
        .expect("two owners link");

    assert_eq!(linked.constructors().len(), 2);
    let owners: BTreeSet<_> = linked
        .constructors()
        .keys()
        .map(|(owner, _)| owner.clone())
        .collect();
    assert_eq!(owners.len(), 2);

    let programs: Vec<_> = linked
        .constructors()
        .values()
        .map(LinkedLiveConstructor::programs)
        .collect();
    assert_ne!(
        programs[0], programs[1],
        "two owners produced one program set, so the owner never reached a leaf",
    );
}

// --- §11.2 symbols ----------------------------------------------------

#[test]
fn every_guide_listed_symbol_role_is_filled_across_the_link() {
    // §11.2's fifteen named roles, as a census rather than a list in a
    // comment. The check reads `GUIDE_LISTED` rather than the census's
    // own `ALL`, so a census that filled every role it invented and none
    // the guide named would fail here.
    //
    // The union across plans is the right level, because §11.3 keeps the
    // two representations' leaf sets disjoint and each emitted bundle
    // carries one: the explicit census cannot fill the private
    // coordinator's role and is not asked to.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");

    assert_eq!(
        link_role_defects(
            linked
                .definitions()
                .iter()
                .map(|((_, plan), census)| (*plan, census))
        ),
        []
    );

    let filled: BTreeSet<LiveLinkRole> = linked
        .definitions()
        .values()
        .flat_map(crate::LiveDefinitionCensus::filled_roles)
        .collect();
    for role in LiveLinkRole::GUIDE_LISTED {
        assert!(filled.contains(role), "the link does not fill {role:?}");
    }

    // And each plan's own program roles land in that plan's own census,
    // which is what keeps the union from hiding a role filled twice by
    // one side and never by the other.
    for ((_, representation), census) in linked.definitions() {
        let roles = census.filled_roles();
        let (coordinator, member) = match representation {
            LiveTransferRepresentationPlan::Explicit => (
                LiveLinkRole::ExplicitCoordinatorProgram,
                LiveLinkRole::ExplicitMemberProgram,
            ),
            LiveTransferRepresentationPlan::PrivateCommitted => (
                LiveLinkRole::PrivateCoordinatorProgram,
                LiveLinkRole::PrivateMemberProgram,
            ),
        };
        assert!(roles.contains(&coordinator), "{representation:?}");
        assert!(roles.contains(&member), "{representation:?}");
    }
}

#[test]
fn a_link_over_one_plan_is_not_required_to_fill_the_other_plans_roles() {
    // §11.6 has the candidate state its plan status precisely so a
    // reader can tell which plans it speaks for. A link over the
    // explicit plan alone is a legitimate artifact, and the role census
    // says so rather than reporting the private plan's programs missing.
    let target = reviewed_target();
    let explicit = single_live_bundle(LiveTransferRepresentationPlan::Explicit, &[0x11; 32]);
    let linked = link_live_candidate(&target, &[explicit], &live_deployment(&target))
        .expect("a one-plan link completes");

    assert_eq!(
        linked.representation_plans(),
        &BTreeSet::from([LiveTransferRepresentationPlan::Explicit]),
    );
    assert_eq!(
        link_role_defects(
            linked
                .definitions()
                .iter()
                .map(|((_, plan), census)| (*plan, census))
        ),
        []
    );
}

#[test]
fn the_constructor_symbol_is_the_one_thing_the_link_itself_settles() {
    // §11.2's live-receipt constructor, and the §10.4 induction's
    // link-time end. Nothing before this layer can settle it — no leaf
    // carries a literal for an owner-parameterized program — so its
    // origin is the link and nothing else in the census shares that
    // origin.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");

    for ((_, representation), census) in linked.definitions() {
        let settled: Vec<_> = census.from_origin(LiveDefinitionOrigin::Link).collect();
        assert_eq!(settled.len(), 1, "{representation:?}");
        assert!(matches!(
            settled[0],
            LiveLinkSymbol::LiveReceiptConstructor { .. }
        ));
    }
}

#[test]
fn the_sighash_profile_is_a_link_symbol_carrying_a_completed_review() {
    // The Wave-5/7 decision stands and is what this still checks: the
    // profile lives at the link and the review does not, so the symbol
    // reports whatever the reviewed contract establishes rather than
    // deciding it. What moved is that answer. It was an incomplete
    // review, carried rather than smoothed over; it is a complete one
    // since the review verdict and the re-typing, and the
    // link reports that for the same reason it reported the other — it
    // reads the contract.
    //
    // All three sites are asserted, because the obligation and the
    // residual are separate lists that could drift apart from the
    // disposition and from each other.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");

    assert!(linked.sighash_profile().is_established());
    assert_eq!(
        *linked.sighash_profile().disposition(),
        OwnerProfileDisposition::Established,
    );
    assert!(
        !linked
            .outstanding_obligations()
            .holds(LiveLinkObligation::SighashProfileUnreviewed),
    );
    assert!(
        !linked
            .residuals()
            .contains(&RecognitionResidual::SighashProfileUnreviewed),
    );

    // The profile reached the census under the reviewed contract's own
    // origin rather than the bundle's or the deployment's.
    for census in linked.definitions().values() {
        let from_review: Vec<_> = census
            .from_origin(LiveDefinitionOrigin::ReviewedContract)
            .collect();
        assert_eq!(from_review, vec![&LiveLinkSymbol::SelectedSighashProfile]);
    }
}

// --- §11.4 taptree ----------------------------------------------------

#[test]
fn each_constructors_tree_is_deterministic_exact_and_within_the_policy() {
    let target = reviewed_target();
    let deployment = live_deployment(&target);
    let linked =
        link_live_candidate(&target, &live_bundles(), &deployment).expect("the link completes");

    for constructor in linked.constructors().values() {
        let tree = constructor.taptree();
        assert_eq!(tree.recipes().len(), 29);
        assert_eq!(
            tree.optimum_route(),
            ExactOptimumRoute::EqualWeightClosedForm,
        );
        assert_eq!(constructor.control_path_depth(), 5);
        assert!(constructor.control_path_depth() <= deployment.maximum_control_path_depth().get());
    }
}

#[test]
fn linking_twice_produces_one_identical_bundle() {
    // Determinism at the level a consumer cares about: the same inputs
    // produce the same linked candidate, by value, because §1.13 mints
    // no digest and the typed value *is* the identity.
    let target = reviewed_target();
    let first = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");
    let second = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes again");
    assert_eq!(first, second);
}

#[test]
fn a_depth_policy_the_candidate_exceeds_refuses_the_whole_link() {
    // §1.11: a refusal returns no partial bundle. The depth policy is
    // the easiest stage to make fail, and what it must not do is return
    // a bundle with one constructor linked and the other missing.
    let target = reviewed_target();
    let shallow = super::live_deployment_at_depth(&target, 4);
    assert!(matches!(
        link_live_candidate(&target, &live_bundles(), &shallow),
        Err(LinkRefusal::LiveTreeDepthExceeded { .. }),
    ));
}

// --- §11.5 carrier closure --------------------------------------------

#[test]
fn the_two_plans_differ_by_exactly_the_conservation_of_the_protocol_asset() {
    // The fact §11.5 is about, read off the compiler's own projections.
    // The explicit plan requires a local carrier for the conservation of
    // the protocol asset in both its sponsor cases; the private plan
    // requires none, because §6.3 makes that equation the target's
    // confidential-transaction rule.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");
    let closure = linked.carrier_closure();

    let explicit = closure
        .plan(LiveTransferRepresentationPlan::Explicit)
        .expect("the explicit plan is linked");
    let private = closure
        .plan(LiveTransferRepresentationPlan::PrivateCommitted)
        .expect("the private plan is linked");

    assert_eq!(explicit.required().len(), 32);
    assert_eq!(private.required().len(), 30);
    assert_eq!(
        closure
            .plan_only()
            .get(&LiveTransferRepresentationPlan::Explicit)
            .map(BTreeSet::len),
        Some(2),
        "the explicit plan requires exactly two carriers the private plan does not",
    );
    assert_eq!(
        closure
            .plan_only()
            .get(&LiveTransferRepresentationPlan::PrivateCommitted)
            .map(BTreeSet::len),
        Some(0),
    );

    // Both execution cases of each plan are covered, which is §11.5's
    // "for every execution case".
    assert_eq!(explicit.cases().len(), 2);
    assert_eq!(private.cases().len(), 2);
}

#[test]
fn the_private_plan_carries_its_value_equation_outside_the_bundle() {
    // §11.5's last sentence, over the case it exists for. The private
    // plan's conservation is an external confidential-value requirement,
    // it is named as such, and no relation-case of that plan is both
    // externally carried and locally reachable.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");

    let private = linked
        .carrier_closure()
        .plan(LiveTransferRepresentationPlan::PrivateCommitted)
        .expect("the private plan is linked");
    assert!(
        private
            .external_evidence()
            .contains(&tapscript::upstream::ExternalEvidenceRole::ConfidentialValueConservation)
    );
    assert!(
        linked
            .unresolved_external_evidence()
            .contains(&tapscript::upstream::ExternalEvidenceRole::ConfidentialValueConservation),
        "the linked bundle drops the external requirement it must carry",
    );

    for key in private.externally_carried() {
        assert!(
            !private.reachable().contains_key(key),
            "an external requirement was reassigned to a local program",
        );
    }

    // The explicit plan does not name it, which is what makes the two
    // censuses a real per-plan comparison rather than one list.
    let explicit = linked
        .carrier_closure()
        .plan(LiveTransferRepresentationPlan::Explicit)
        .expect("the explicit plan is linked");
    assert!(
        !explicit
            .external_evidence()
            .contains(&tapscript::upstream::ExternalEvidenceRole::ConfidentialValueConservation)
    );
}

#[test]
fn a_plan_whose_own_tree_lacks_a_carrier_starves_rather_than_borrowing() {
    // §11.5's headline, made reachable. The private plan's committed
    // leaves are emptied of coordinators while the explicit plan's are
    // left whole, so every case the private plan requires is reachable
    // in the explicit tree and in no tree of its own. A closure computed
    // over the union would have reported them carried.
    let bundles = live_bundles();
    let plan = bundles[0].plan().clone();

    let mut emitted = std::collections::BTreeMap::new();
    let mut committed = std::collections::BTreeMap::new();
    for bundle in &bundles {
        let representation = bundle.representation();
        let leaves: BTreeSet<LiveTransferLeafRole> = bundle.leaves().keys().copied().collect();
        emitted.insert(representation, leaves.clone());
        let kept = if representation == LiveTransferRepresentationPlan::PrivateCommitted {
            leaves
                .into_iter()
                .filter(|leaf| leaf.program_role() != LiveProgramRole::Coordinator)
                .collect()
        } else {
            leaves
        };
        committed.insert(representation, kept);
    }

    assert!(matches!(
        crate::live_carrier::close_live(&plan, &emitted, &committed),
        Err(LinkRefusal::PlanStarvedOfCarrier {
            starved: LiveTransferRepresentationPlan::PrivateCommitted,
            reachable_in: LiveTransferRepresentationPlan::Explicit,
            ..
        }),
    ));
}

// --- §11.6 the linked candidate ---------------------------------------

#[test]
fn the_linked_candidate_retains_every_item_section_eleven_six_lists() {
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");

    // 1 exact scope, 2 exact target projection, 3 exact shape bounds.
    assert_eq!(linked.plan().representations().count(), 2);
    assert_eq!(linked.contract(), reviewed_target().definition().version());
    assert_eq!(linked.abi_handoff().bounds().receipt_inputs(), 3);
    assert_eq!(linked.abi_handoff().bounds().receipt_outputs(), 3);
    assert_eq!(linked.abi_handoff().bounds().sponsor_inputs(), 1);
    // 4 plan status, 5 linked constructors, 6 linked programs.
    assert_eq!(linked.representation_plans().len(), 2);
    assert_eq!(linked.constructors().len(), 2);
    assert_eq!(
        linked
            .constructors()
            .values()
            .map(|constructor| constructor.programs().len())
            .sum::<usize>(),
        58,
    );
    // 7 selected sighash profile.
    assert!(linked.sighash_profile().profile().required().count() > 0);
    // 8 concrete placements.
    assert_eq!(linked.abi_handoff().family_ranges().len(), 27);
    assert!(!linked.carrier_closure().plans().is_empty());
    // 9 ABI handoff.
    assert!(!linked.abi_handoff().witness_roles().is_empty());
    assert!(!linked.abi_handoff().owed().is_empty());
    // 10 exact resource formulas.
    assert_eq!(linked.formulas().len(), 4);
    // 11 unresolved external evidence.
    assert!(!linked.unresolved_external_evidence().is_empty());
    // 12 outstanding burn and redemption lifecycle.
    assert_eq!(linked.outstanding_lifecycle().outstanding().get(), 2);
    // 13 candidate-only status.
    assert_eq!(linked.status(), LinkedArtifactStatus::Prototype);
}

#[test]
fn the_resource_formulas_are_exact_at_every_measured_shape() {
    // §11.6's *exact* resource formulas. Where a model exists it must
    // reproduce every measurement exactly; where none does, the exact
    // table stands alone rather than a model that is right about most of
    // them.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");

    for formulas in linked.formulas().values() {
        for formula in formulas.values() {
            assert!(!formula.measurements().is_empty());
            for (shape, measure) in formula.measurements() {
                if let Some(predicted) = crate::predict_live_model(formula.model(), *shape) {
                    assert_eq!(
                        predicted,
                        i64::try_from(*measure).expect("a linked figure fits"),
                        "the model misses {shape:?} for {:?}/{:?}",
                        formula.representation(),
                        formula.dimension(),
                    );
                }
            }
        }
    }
}

#[test]
fn the_script_byte_formula_really_does_vary_with_the_shape() {
    // A guard against a formula that is exact because everything is
    // constant. The coordinator's script bytes must differ across the
    // shape set, or the exactness above would be saying nothing.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");

    let formula = linked
        .formulas()
        .get(&(
            LiveTransferRepresentationPlan::Explicit,
            LiveProgramRole::Coordinator,
        ))
        .and_then(|formulas| formulas.get(&ResourceDimension::ScriptBytes))
        .expect("the explicit coordinator charges script bytes");

    let figures: BTreeSet<_> = formula.measurements().values().copied().collect();
    assert!(
        figures.len() > 1,
        "every shape's coordinator is the same size, so the formula measures nothing",
    );
}

#[test]
fn no_linked_figure_disagrees_with_the_pre_link_witness_statement() {
    // The one dimension carried across rather than recomputed: a
    // program that scheduled from §10.2's one-item precondition still
    // does, so the linked and pre-link witness statements must agree.
    let target = reviewed_target();
    let bundles = live_bundles();
    let linked =
        link_live_candidate(&target, &bundles, &live_deployment(&target)).expect("the link");

    for bundle in &bundles {
        let owner = OwnerParameter::new(bundle.constructor().owner().clone());
        let constructor = linked
            .constructor(&owner, bundle.representation())
            .expect("each bundle linked");
        for (leaf, pre_link) in bundle.leaves() {
            let linked_program = constructor.program(*leaf).expect("each leaf linked");
            assert_eq!(
                linked_program.charged(ResourceDimension::InitialStackItems),
                pre_link
                    .resources()
                    .get(&ResourceDimension::InitialStackItems)
                    .copied(),
            );
        }
    }
}

#[test]
fn the_control_path_depth_the_bundle_owed_the_linker_is_now_charged() {
    // The emitted bundle names `ControlPathDepth` as
    // `ResourceObligation::Linker`. This is the wave that owes it, and
    // the committed tree settles it — per leaf, because a spender
    // carries one leaf's control block and no other.
    let target = reviewed_target();
    let bundles = live_bundles();
    assert_eq!(
        bundles[0]
            .outstanding_dimensions()
            .get(&ResourceDimension::ControlPathDepth),
        Some(&ResourceObligation::Linker),
    );

    let linked =
        link_live_candidate(&target, &bundles, &live_deployment(&target)).expect("the link");
    for constructor in linked.constructors().values() {
        let depths = crate::control_path_depths(constructor.taptree());
        assert_eq!(depths.len(), 29);
        assert!(depths.values().all(|depth| *depth == 4 || *depth == 5));
    }
}

// --- §10.4's induction end --------------------------------------------

#[test]
fn the_induction_step_names_what_linking_established_and_what_remains() {
    // The wave the induction was waiting on. What moved: for each
    // (owner, representation) the committed tree exists and is the one a
    // destination's version check points at. What did not: the taproot
    // output key and §12.3's destination table, which are two separate
    // things and are reported as two.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");
    let step = linked.induction_step();

    assert_eq!(step.established().len(), 2);
    for ((owner, representation), placement) in step.established() {
        assert_eq!(placement.owner(), owner);
        assert_eq!(placement.representation(), *representation);
        assert_eq!(placement.committed_leaves().len(), 29);
    }

    assert_eq!(
        step.outstanding(),
        &BTreeSet::from([
            LiveLinkObligation::TaprootOutputKeyUndischarged,
            LiveLinkObligation::DestinationConstructorTableUndischarged,
        ]),
    );

    // Re-scoped, not cleared. A *program* still cannot establish a
    // destination's constructor bytes, so the residual stands on both
    // ends and the link says which part of its reason stopped applying.
    assert_eq!(
        step.residual(),
        RecognitionResidual::LinkedDestinationConstructorIdentity,
    );
    assert!(
        linked
            .residuals()
            .contains(&RecognitionResidual::LinkedDestinationConstructorIdentity),
    );
}

#[test]
fn a_linked_bundle_always_owes_something_and_mints_no_digest() {
    // §1.12 and §1.13 together. The obligation set is structurally
    // non-empty, and there is no accessor anywhere on the linked bundle
    // returning a hash — which is checked here by the absence of any
    // route to one rather than by reading a field back.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");

    // Four, not five: the sighash review left the set when the review
    // verdict and the re-typing established every
    // dimension the profile requires. The count is asserted as a literal
    // so that an obligation leaving quietly fails here.
    assert_eq!(linked.outstanding_obligations().count().get(), 4);
    assert!(
        linked
            .outstanding_obligations()
            .holds(LiveLinkObligation::TaprootOutputKeyUndischarged),
    );
    assert!(
        linked
            .outstanding_obligations()
            .holds(LiveLinkObligation::DestinationConstructorTableUndischarged),
    );
    assert!(
        linked
            .outstanding_obligations()
            .holds(LiveLinkObligation::OwnerKeyCurvePointMembershipUnverified),
    );
    assert!(
        linked
            .residuals()
            .contains(&RecognitionResidual::OwnerKeyCurvePointMembership),
    );
    assert!(
        linked
            .residuals()
            .contains(&RecognitionResidual::FieldFormSettledOnlyOnTheTarget),
    );
}

// --- What the linker extension makes expressible ----------------------

#[test]
fn the_artifacts_the_flipped_mutation_cases_needed_now_exist() {
    // The evidence behind Wave 8's flip of
    // [`tapscript::MutationResidual`]. Four constructor mutation cases
    // said they were expressible only in the linked taptree and that
    // §11's linker extension was not built. It is, and this is what that
    // means concretely rather than as an assertion in a census:
    //
    // - `OwnerBytesReplacedInTheLinkedOutput` needs a linked program
    //   whose owner bytes could be replaced. Every leaf carries the
    //   owner as a substituted symbol, so there is one;
    // - `InternalKeyReplacedWithASpendableOne` needs a linked
    //   constructor carrying a resolved internal key. Each does;
    // - `BurnLeafAddedToTheTaptree` needs a committed taptree with a
    //   leaf census to add to. Each constructor has one;
    // - `RepresentationSwappedInTheLinkedTree` needs two representations'
    //   leaves to swap between. Both are linked, and their leaf sets are
    //   disjoint.
    //
    // What none of them has is a transaction, which is why every one of
    // them now names the candidate ABI instead.
    let target = reviewed_target();
    let linked = link_live_candidate(&target, &live_bundles(), &live_deployment(&target))
        .expect("the link completes");

    let mut leaf_sets = Vec::new();
    for constructor in linked.constructors().values() {
        assert_ne!(constructor.internal_key().bytes().len(), 0);
        assert_ne!(constructor.taptree().recipes().len(), 0);

        let owner_bearing = constructor
            .substituted()
            .values()
            .filter(|symbols| symbols.contains(&tapscript::LiveBundleSymbol::OwnerPublicKey))
            .count();
        assert_eq!(
            owner_bearing, 29,
            "no linked leaf carries the owner, so there is nothing to replace",
        );

        leaf_sets.push(
            constructor
                .taptree()
                .recipes()
                .keys()
                .copied()
                .collect::<BTreeSet<LiveTransferLeafRole>>(),
        );
    }

    assert_eq!(leaf_sets.len(), 2);
    assert!(
        leaf_sets[0].is_disjoint(&leaf_sets[1]),
        "the two representations' committed leaves overlap, so a swap would be no swap",
    );
}

// --- Refusals ---------------------------------------------------------

#[test]
fn an_empty_offering_has_nothing_to_link() {
    let target = reviewed_target();
    assert_eq!(
        link_live_candidate(&target, &[], &live_deployment(&target)),
        Err(LinkRefusal::NoLiveBundleOffered),
    );
}

#[test]
fn one_owner_and_representation_claimed_twice_is_a_refusal() {
    // §7.6 gives each owner one constructor per representation, so a
    // second claim on one key is two constructors for one destination
    // and no way to say which a linked output is under.
    let target = reviewed_target();
    let bundle = single_live_bundle(LiveTransferRepresentationPlan::Explicit, &[0x11; 32]);
    assert_eq!(
        link_live_candidate(
            &target,
            &[bundle.clone(), bundle],
            &live_deployment(&target),
        ),
        Err(LinkRefusal::DuplicateLinkedConstructor {
            representation: LiveTransferRepresentationPlan::Explicit,
        }),
    );
}
