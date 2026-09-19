//! The maturity link's sources, bound from typed fixtures.
//!
//! The plan, the record, the operator binding and the fixture lead window
//! are the shared maturity fixtures in [`crate::tests`], which is also
//! where the argument that they are real artifacts and that their bytes
//! are test material lives. This file is about what binding them
//! establishes.
//!
//! # Raw bytes are refused by the compiler, not by a test
//!
//! That no source can arrive as bytes is a statement about types, so it
//! is checked where the type checker can fail it: the `compile_fail`
//! doctests on [`StateLinkDeploymentParameters::bind`] offer a byte
//! vector in place of the plan and a value map in place of the record. An
//! assertion here could only restate the claim in prose, and prose cannot
//! fail on the day a byte parameter is added.
//!
//! # Two arms of the reused check are not reachable from here
//!
//! [`crate::LinkRefusal::OperatorKeyMismatch`] cannot be raised through
//! the bridge, which offers the binding's own key, and
//! [`crate::LinkRefusal::InvalidOperatorProfile`] cannot be raised while
//! one reviewed definition exists, because the profile's pin and the
//! bridge's revision are both read from it. The second is exercised where
//! it is reachable: `check_refuses_stale_revision_v1` in the
//! operator-deployment tests offers the earlier revision to
//! [`crate::OperatorDeploymentBinding::check`] directly.

use std::collections::BTreeSet;

use architecture::OperationId;
use tapscript::StateExternalEvidenceRole;
use tapscript::upstream::Cycle;
use target_elements::TargetContractVersion;

use crate::tests::{
    binding, bridge, depth, fixture_lead_bounds, identity, plan, record, reviewed_target,
};
use crate::{LinkRefusal, StateLeadBoundOrigin, StateLinkDeploymentParameters};

#[test]
fn binding_returns_every_supplied_source_and_the_reviewed_revision() {
    let bound = bridge();
    assert_eq!(bound.plan(), &plan());
    assert_eq!(bound.lead_bounds(), fixture_lead_bounds());
    assert_eq!(bound.lead_bounds().origin(), StateLeadBoundOrigin::Fixture);
    assert_eq!(bound.lead_bounds().bounds().minimum(), Cycle::new(2));
    assert_eq!(bound.lead_bounds().bounds().maximum(), Cycle::new(4));
    assert_eq!(bound.identity(), &identity(0x11, 0x22));
    assert_eq!(bound.operator(), &binding());
    assert_eq!(bound.maximum_control_path_depth(), depth());
    assert_eq!(bound.revision(), reviewed_target().definition().version());
    assert_eq!(bound.revision(), TargetContractVersion::V2);
    assert_eq!(bound.operator().capability_revision(), bound.revision());
}

#[test]
fn the_deployment_facts_are_exactly_the_three_deployment_side_roles() {
    assert_eq!(
        bridge().deployment_facts(),
        &BTreeSet::from([
            StateExternalEvidenceRole::SubstrateConservation,
            StateExternalEvidenceRole::SingletonNonReissuable,
            StateExternalEvidenceRole::SingletonIssuedUnderConstructor,
        ])
    );
}

// The two facts the reduced leaf rests on and does not check. Naming them
// beside the binding is what sends a reader to deployment records for
// them; the filter is a filter, so the report-layer role the record also
// carries stays out of the binding.
#[test]
fn both_singleton_facts_appear_in_the_binding_and_freshness_does_not() {
    let bound = bridge();
    let facts = bound.deployment_facts();
    assert!(facts.contains(&StateExternalEvidenceRole::SingletonNonReissuable));
    assert!(facts.contains(&StateExternalEvidenceRole::SingletonIssuedUnderConstructor));

    let composed = record();
    let external = &composed.metadata().external;
    assert!(external.contains(&StateExternalEvidenceRole::CurrentStateRootFreshness));
    assert!(!facts.contains(&StateExternalEvidenceRole::CurrentStateRootFreshness));
}

#[test]
fn bind_refuses_an_identity_other_than_the_operators() {
    let other = identity(0x11, 0x44);
    assert_eq!(
        StateLinkDeploymentParameters::bind(
            &reviewed_target(),
            plan(),
            fixture_lead_bounds(),
            other.clone(),
            binding(),
            depth(),
            &record(),
        ),
        Err(LinkRefusal::OperatorDeploymentMismatch {
            bound: Box::new(identity(0x11, 0x22)),
            offered: Box::new(other),
        })
    );
}

#[test]
fn the_bound_plan_is_the_announcement_plan() {
    assert_eq!(bridge().plan().operation(), OperationId::AnnounceMaturity);
}
