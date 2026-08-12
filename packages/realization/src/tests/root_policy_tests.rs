//! Root-use policy against actual observed root effects (SR2-02).
//!
//! The two Phase-2 pilots are root-free, so these cases are synthetic.
//! They are named after the RESV shapes they anticipate: an ordinary
//! redemption succeeds the reserve root, and a sealing redemption
//! terminates it, both under the one `SuccessionOrTermination` policy
//! the architecture declares for `REDEEM`.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{OperationId, ProjectionId, RootId, RootUse};

use crate::{
    ObservedCanonicalPartition, ObservedRootEffect, ObservedRootEffectKind, OperationObservation,
    evaluate::root_policy_holds,
};

/// A root-free observation carrying only the given root effects.
fn observation(effects: Vec<ObservedRootEffect>) -> OperationObservation {
    OperationObservation {
        operation: OperationId::Redeem,
        objects: Vec::new(),
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::new(),
        canonical_partition: ObservedCanonicalPartition::default(),
        open_flows: Vec::new(),
        root_effects: effects,
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::new(),
    }
}

fn resv(effect: ObservedRootEffectKind) -> Vec<ObservedRootEffect> {
    vec![ObservedRootEffect {
        root: RootId::Resv,
        effect,
    }]
}

fn policy(use_kind: RootUse) -> BTreeMap<RootId, RootUse> {
    BTreeMap::from([(RootId::Resv, use_kind)])
}

#[test]
fn succession_policy_admits_a_succession() {
    assert!(root_policy_holds(
        &policy(RootUse::Succession),
        &observation(resv(ObservedRootEffectKind::Succession)),
    ));
}

#[test]
fn succession_policy_rejects_a_termination() {
    assert!(!root_policy_holds(
        &policy(RootUse::Succession),
        &observation(resv(ObservedRootEffectKind::Termination)),
    ));
}

#[test]
fn succession_or_termination_admits_normal_resv_succession() {
    // The regression this finding names: an ordinary, non-sealing
    // redemption succeeds RESV under a policy that explicitly permits
    // succession. Comparing the effect against the policy value made
    // this fail.
    assert!(root_policy_holds(
        &policy(RootUse::SuccessionOrTermination),
        &observation(resv(ObservedRootEffectKind::Succession)),
    ));
}

#[test]
fn succession_or_termination_admits_sealing_resv_termination() {
    assert!(root_policy_holds(
        &policy(RootUse::SuccessionOrTermination),
        &observation(resv(ObservedRootEffectKind::Termination)),
    ));
}

#[test]
fn forbidden_policy_rejects_every_effect() {
    for effect in [
        ObservedRootEffectKind::Succession,
        ObservedRootEffectKind::Termination,
    ] {
        assert!(
            !root_policy_holds(&policy(RootUse::Forbidden), &observation(resv(effect))),
            "a forbidden root must admit no effect, but admitted {effect:?}",
        );
        assert!(
            !effect.permitted_by(RootUse::Forbidden),
            "compatibility must reject {effect:?} directly as well",
        );
    }
}

#[test]
fn forbidden_policy_admits_the_absence_of_any_effect() {
    assert!(root_policy_holds(
        &policy(RootUse::Forbidden),
        &observation(Vec::new()),
    ));
}

#[test]
fn a_required_root_effect_may_not_be_omitted() {
    // Separating effect from policy must not weaken the requirement
    // direction: a non-forbidden policy still demands an actual effect.
    for use_kind in [RootUse::Succession, RootUse::SuccessionOrTermination] {
        assert!(
            !root_policy_holds(&policy(use_kind), &observation(Vec::new())),
            "{use_kind:?} must require an observed effect",
        );
    }
}

#[test]
fn a_root_may_carry_only_one_effect() {
    let doubled = vec![
        ObservedRootEffect {
            root: RootId::Resv,
            effect: ObservedRootEffectKind::Succession,
        },
        ObservedRootEffect {
            root: RootId::Resv,
            effect: ObservedRootEffectKind::Termination,
        },
    ];

    assert!(!root_policy_holds(
        &policy(RootUse::SuccessionOrTermination),
        &observation(doubled),
    ));
}
