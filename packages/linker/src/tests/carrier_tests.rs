//! Relation-carrier closure against the committed tree (§14.6).

use std::collections::BTreeSet;

use tapscript::{ConcreteCarrierSite, LeafRole, ProgramRole};

use crate::carrier::close;
use crate::tests::{deployment, relocatable_bundle, reviewed_target};
use crate::{LinkRefusal, SelfCommitmentStrategy, link_candidate};

/// Every leaf the bundle emitted.
fn every_leaf() -> BTreeSet<LeafRole> {
    relocatable_bundle()
        .constructor()
        .leaves()
        .keys()
        .copied()
        .collect()
}

#[test]
fn the_three_censuses_are_one_set_and_every_case_stays_reachable() {
    // §14.6's comparison: what the compiler required, what the backend
    // emitted, and what the tree leaves reachable. The first two must
    // be the same set — a relation that vanished at the boundary is
    // what §1.3 forbids — and every case must still have a site with a
    // committed program behind it.
    let bundle = relocatable_bundle();
    let closure = close(&bundle, &every_leaf()).expect("the carrier census closes");

    assert_eq!(closure.required(), closure.emitted());
    assert_eq!(closure.required().len(), 30);
    assert_eq!(closure.reachable().len(), 30);
    for (key, sites) in closure.reachable() {
        assert!(!sites.is_empty(), "{key:?} has no reachable site");
    }
}

#[test]
fn every_case_of_this_candidate_is_carried_by_exactly_one_site() {
    // A fact about this candidate rather than about linking in
    // general, and worth writing down because it makes the last clause
    // of §14.6 load-bearing everywhere: there is no relation-case with
    // a spare carrier, so removing any carrying program breaks
    // something.
    let bundle = relocatable_bundle();
    let closure = close(&bundle, &every_leaf()).expect("the carrier census closes");

    assert_eq!(closure.uniquely_carried().len(), 30);
    assert_eq!(closure.uniquely_carried().len(), closure.required().len());
}

#[test]
fn removing_one_coordinator_leaf_breaks_a_uniquely_carrying_program() {
    // The check that makes the closure a check. A tree missing one
    // coordinator leaf is a tree in which some relation-case's only
    // carrier is gone, and §14.6 requires that to be a refusal rather
    // than a smaller bundle.
    let bundle = relocatable_bundle();
    let mut committed = every_leaf();
    let removed = *committed
        .iter()
        .find(|leaf| leaf.program_role() == ProgramRole::Coordinator)
        .expect("the candidate has a coordinator leaf");
    committed.remove(&removed);

    assert!(matches!(
        close(&bundle, &committed),
        Err(LinkRefusal::UniqueCarrierRemoved { .. })
    ));
}

#[test]
fn a_tree_with_no_member_leaf_leaves_a_member_carried_case_unreachable() {
    // The other half: a site whose whole program role is absent. Every
    // member leaf removed at once, so the failure is reachability
    // rather than the uniqueness clause.
    let bundle = relocatable_bundle();
    let committed: BTreeSet<LeafRole> = every_leaf()
        .into_iter()
        .filter(|leaf| leaf.program_role() != ProgramRole::Member)
        .collect();

    assert!(matches!(
        close(&bundle, &committed),
        Err(LinkRefusal::UnreachableRelationCarrier(_) | LinkRefusal::UniqueCarrierRemoved { .. })
    ));
}

#[test]
fn an_empty_tree_carries_nothing() {
    let bundle = relocatable_bundle();
    assert!(close(&bundle, &BTreeSet::new()).is_err());
}

#[test]
fn the_linked_closure_names_only_sites_this_candidate_has_programs_for() {
    // The sponsor region carries no protocol leaf in this candidate —
    // §12.9 has the coordinator prove the region instead — so no
    // placement may name a site outside the two leaf roles and the two
    // non-program sites.
    let target = reviewed_target();
    let linked = link_candidate(
        &target,
        &relocatable_bundle(),
        &deployment(
            &target,
            SelfCommitmentStrategy::ExternallyAuthenticatedCommitment,
        ),
    )
    .expect("the demonstration link completes");

    let admitted = BTreeSet::from([
        ConcreteCarrierSite::CoordinatorLeaf,
        ConcreteCarrierSite::MemberLeaf,
        ConcreteCarrierSite::BundleStructure,
        ConcreteCarrierSite::OutsideBundle,
    ]);
    for (key, sites) in linked.carrier_closure().reachable() {
        for site in sites {
            assert!(admitted.contains(site), "{key:?} names an unadmitted site");
        }
    }
}
