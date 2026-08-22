//! Recomputation oracles for the live-transfer target-operation plan
//! (Guide-13 §5, §6, §8.3, §19).
//!
//! Every census the plan publishes is checked against the *analyzed
//! program*, and every §5 projection against the *realization
//! declarations* the analysis retained — never against the construction
//! that produced the plan. A test that re-ran the plan's own projection
//! helpers and compared the result would prove only that the module is
//! deterministic, which is not the claim §8.1 makes.
//!
//! The §5 expectations below are therefore written as the guide's own
//! fixed values, spelled out here rather than read from the plan: an
//! oracle that asked the plan what the contract said could not tell a
//! correct plan from a plan that had adopted a drifted realization.
//!
//! The corruption oracles run the other way. A validated plan is
//! assembled, one published field is damaged in a way a defect in the
//! join could produce, and the independent validator is required to
//! reject it with the typed variant that names the damage. §8.3 lists
//! the damages; each row there has a test here.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    AssetId, DeltaKind, ObjectId, OpenFlowKind, OperationId, ProjectionId, ProjectionRule, RootId,
    RootUse,
};
use realization::{
    CardinalityMaximum, ConstructibilityClass, Count, ExpectedCanonicalDelta, ObservedSide,
    Relation, RelationId, RelationKind, RelationSubject, RepresentationMode,
};

use super::bound_input;
use crate::{
    CompileError,
    analyzed::{ScopedAnalyzedProgram, analyze_scoped_program},
    capability::RequiredCapability,
    live_transfer_plan::{
        DeferredRepresentation, LiveTransferClause, LiveTransferRepresentationPlan,
        RepresentationDeferralGround, ValidatedLiveTransferOperationPlan,
        plan_live_transfer_target_operation, validate_live_transfer_plan,
    },
    placement::PlacementSearchLimits,
    sponsor_region::OrdinaryLbtcRole,
};

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

/// The two pilot scopes that analyze the live transfer.
fn scopes() -> Vec<Vec<OperationId>> {
    vec![
        vec![OperationId::TransferLive],
        vec![OperationId::CompactAsh, OperationId::TransferLive],
    ]
}

/// One plan and the analyzed program it must agree with.
struct Subject {
    analyzed: ScopedAnalyzedProgram,
    plan: ValidatedLiveTransferOperationPlan,
}

fn subject(scope: &[OperationId]) -> Subject {
    let input = bound_input(scope);

    Subject {
        analyzed: analyze_scoped_program(&input, limits()).expect("analysis"),
        plan: plan_live_transfer_target_operation(&input, limits()).expect("plan"),
    }
}

/// The combined pilot scope, where the compact-ASH representation
/// crosses the live-transfer one and four plans survive.
fn corruption_subject() -> Subject {
    subject(&[OperationId::CompactAsh, OperationId::TransferLive])
}

/// The value-conservation relation of the live transfer, by identity.
fn conservation() -> RelationId {
    RelationId::new(
        OperationId::TransferLive,
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    )
}

/// A relation of the other pilot, which this operation never owns.
fn foreign_relation() -> RelationId {
    RelationId::new(
        OperationId::CompactAsh,
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    )
}

// --- §5: the contract projections say what the guide fixes ---

#[test]
fn the_class_projection_is_receipt_l_closed_against_every_other_family() {
    for scope in scopes() {
        let plan = subject(&scope).plan;
        let class = plan.class();

        // §5.1, §5.4: one protocol class, on both sides.
        assert_eq!(class.protocol(), ObjectId::ReceiptLive, "{scope:?}");
        assert_eq!(class.sponsor(), ObjectId::PlainLbtc, "{scope:?}");
        assert_eq!(
            class.admitted().collect::<Vec<_>>(),
            [ObjectId::ReceiptLive, ObjectId::PlainLbtc],
            "{scope:?}",
        );

        // §5.4's forbidden list, and every other family the architecture
        // knows: the census is the complement of the admitted set, so no
        // family can be forbidden by omission.
        for object in [
            ObjectId::ReceiptTimeLocked,
            ObjectId::Ash,
            ObjectId::DistributionVault,
            ObjectId::DistributionControl,
            ObjectId::DepositEntitlement,
            ObjectId::DepositRequest,
            ObjectId::State,
            ObjectId::Resv,
        ] {
            assert!(class.forbids(object), "{object:?} in {scope:?}");
        }

        assert_eq!(
            class.forbidden().count(),
            ObjectId::ALL.len() - 2,
            "{scope:?}",
        );
        assert!(!class.forbids(ObjectId::ReceiptLive) && !class.forbids(ObjectId::PlainLbtc));
    }
}

#[test]
fn the_owner_projection_binds_authorization_and_constructibility_to_one_family() {
    let plan = corruption_subject().plan;
    let owner = plan.owner();

    // §5.5, §8.2: every input owner. Two relations over one family is
    // what carries the quantifier the declarations do not spell.
    assert_eq!(owner.object(), ObjectId::ReceiptLive);
    assert_eq!(
        owner.class(),
        ConstructibilityClass::OwnersOf {
            object: ObjectId::ReceiptLive,
        },
    );
    assert_eq!(owner.authorization().operation(), OperationId::TransferLive);
    assert_eq!(owner.authorization().kind(), RelationKind::Authorization);
    assert_eq!(
        owner.constructibility().kind(),
        RelationKind::Constructibility,
    );
}

#[test]
fn the_value_projection_conserves_exact_u_laterally_between_receipts() {
    let plan = corruption_subject().plan;
    let value = plan.value();

    // §5.1: the exact conservation relation, receipts to receipts.
    assert_eq!(value.asset(), AssetId::U);
    assert_eq!(
        value.objects(ObservedSide::Input).collect::<Vec<_>>(),
        [ObjectId::ReceiptLive],
    );
    assert_eq!(
        value.objects(ObservedSide::Output).collect::<Vec<_>>(),
        [ObjectId::ReceiptLive],
    );

    // §5.6: one lateral flow, no issuance, no destruction.
    assert_eq!(
        value.flow().cloned().collect::<Vec<_>>(),
        [ExpectedCanonicalDelta {
            asset: AssetId::U,
            kind: DeltaKind::Lateral,
            destruction_tag: None,
        }],
    );
    assert!(!value.issues() && !value.destroys());

    // §5.1: at least one receipt each side, bounded by an
    // architecture-owned bound whose candidate assignment is not this
    // wave's to make.
    for side in [ObservedSide::Input, ObservedSide::Output] {
        let cardinality = value.cardinality(side);

        assert_eq!(cardinality.minimum(), Count::ONE, "{side:?}");
        assert!(!cardinality.is_optional(), "{side:?}");
        assert!(
            matches!(cardinality.maximum(), CardinalityMaximum::Bound(_)),
            "{side:?} maximum stays an architecture bound",
        );
    }
}

#[test]
fn the_sponsor_projection_carries_the_region_and_no_amount() {
    let plan = corruption_subject().plan;
    let sponsor = plan.sponsor();

    // §5.8: the fee sponsor is the only open flow, so ordinary L-BTC is
    // exactly the sponsor region.
    assert_eq!(
        sponsor.open_flows().collect::<Vec<_>>(),
        [OpenFlowKind::FeeSponsor],
    );
    assert_eq!(sponsor.region(), OrdinaryLbtcRole::SponsorRegion);
    assert_eq!(sponsor.object(), ObjectId::PlainLbtc);

    // §1.9: at most one envelope; the envelope and its change are both
    // optional, and the change is one output at most.
    assert_eq!(sponsor.envelope_maximum(), Count::ONE);
    assert!(sponsor.is_optional());
    assert!(sponsor.change_cardinality().is_optional());
    assert_eq!(
        sponsor.change_cardinality().maximum(),
        CardinalityMaximum::Exact(Count::ONE),
    );

    // §1.9: whole-transaction conservation over the substrate asset
    // stays a relation, never an amount this plan reads.
    assert_eq!(
        sponsor.substrate_conservation().kind(),
        RelationKind::SubstrateConservation,
    );
    assert_eq!(sponsor.isolation().kind(), RelationKind::SponsorIsolation);
}

#[test]
fn every_architecture_root_is_forbidden_and_only_the_certificate_is_required() {
    let plan = corruption_subject().plan;

    // §5.7: every root forbidden, over the complete root census.
    assert!(plan.roots().forbids_every_root());
    assert_eq!(plan.roots().expected().count(), RootId::ALL.len());

    for root in RootId::ALL.iter().copied() {
        assert_eq!(plan.roots().root_use(root), Some(RootUse::Forbidden));
    }

    // §5.7: the transition certificate is required; burn, clear, and
    // distribution residue are forbidden.
    let certificate = plan.certificate();

    assert_eq!(
        certificate.required().collect::<Vec<_>>(),
        [ProjectionId::TransitionCertificate],
    );
    assert_eq!(
        certificate.forbidden().collect::<Vec<_>>(),
        [
            ProjectionId::BurnEvent,
            ProjectionId::ClearEvent,
            ProjectionId::DistributionResidue,
        ],
    );
    assert_eq!(certificate.expected().count(), ProjectionId::ALL.len());
    assert_eq!(
        certificate.rule(ProjectionId::BurnEvent),
        Some(ProjectionRule::Forbidden),
    );
}

#[test]
fn the_lifecycle_closure_implements_transfer_and_leaves_burn_and_redeem_outstanding() {
    for scope in scopes() {
        let plan = subject(&scope).plan;

        // §1.12, §8.2: the live-receipt candidate is lifecycle-
        // incomplete and says which exits are missing.
        assert_eq!(
            plan.lifecycle().implemented().collect::<Vec<_>>(),
            [OperationId::TransferLive],
            "{scope:?}",
        );
        assert_eq!(
            plan.lifecycle().outstanding().collect::<Vec<_>>(),
            [OperationId::Redeem, OperationId::Burn],
            "{scope:?}",
        );
        assert!(!plan.lifecycle().release_complete(), "{scope:?}");
    }
}

// --- §6: the representation alternatives and the deferrals ---

#[test]
fn both_admitted_representations_are_planned_and_the_deferrals_are_typed() {
    let plan = corruption_subject().plan;
    let policy = plan.representation();

    // §6.1: exactly the two admitted modes, and the realization's own
    // approved set is what they were checked against.
    assert_eq!(
        policy.admitted().collect::<Vec<_>>(),
        [
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ],
    );
    assert_eq!(
        policy.approved().collect::<Vec<_>>(),
        [
            RepresentationMode::Explicit,
            RepresentationMode::PrivateCommitted,
        ],
    );

    // §6.1, §6.5: what is not admitted is recorded with the ground it
    // rests on, never left as silence.
    assert_eq!(
        policy.deferral(DeferredRepresentation::PublicCommittedMode),
        Some(RepresentationDeferralGround::UnapprovedMode),
    );
    assert_eq!(
        policy.deferral(DeferredRepresentation::MixedComposition),
        Some(RepresentationDeferralGround::NoPerReferenceVariable),
    );
    assert_eq!(policy.deferred().count(), DeferredRepresentation::ALL.len());

    // Both admitted plans really are planned.
    for representation in LiveTransferRepresentationPlan::ALL.iter().copied() {
        let projection = plan.projection(representation).expect("a projection");

        assert_eq!(projection.plan(), representation);
        assert!(projection.relations().count() > 0);
        assert!(projection.coverage().count() > 0);
    }
}

#[test]
fn the_admitted_and_deferred_representations_partition_the_mode_vocabulary() {
    // The partition is a property of two exhaustive matches rather than
    // of a list: every mode is an admitted plan or it is not, and the
    // one that is not is the mode the deferral census names.
    for representation in LiveTransferRepresentationPlan::ALL.iter().copied() {
        assert_eq!(
            LiveTransferRepresentationPlan::of(representation.mode()),
            Some(representation),
        );
    }

    assert_eq!(
        LiveTransferRepresentationPlan::of(RepresentationMode::PublicCommitted),
        None,
        "the deferred mode has no admitted plan",
    );
    assert_eq!(
        DeferredRepresentation::PublicCommittedMode.ground(),
        RepresentationDeferralGround::UnapprovedMode,
    );
}

#[test]
fn the_representations_agree_on_every_relation_and_differ_only_on_conservation() {
    let plan = corruption_subject().plan;
    let explicit = plan
        .projection(LiveTransferRepresentationPlan::Explicit)
        .expect("the explicit projection");
    let private = plan
        .projection(LiveTransferRepresentationPlan::PrivateCommitted)
        .expect("the private projection");

    // §6.6, §1.2: the relation censuses are equal.
    let census = |projection: &crate::live_transfer_plan::LiveTransferRepresentationProjection| {
        projection
            .relations()
            .map(|requirement| requirement.relation.clone())
            .collect::<BTreeSet<_>>()
    };

    assert_eq!(census(explicit), census(private));

    // §19.4: exactly the value-conservation relation differs, and the
    // difference is the pair of capabilities the guide names.
    let diverging = census(explicit)
        .into_iter()
        .filter(|relation| {
            let left = explicit.relation(relation).expect("explicit requirement");
            let right = private.relation(relation).expect("private requirement");

            left.proof != right.proof || left.required_capabilities != right.required_capabilities
        })
        .collect::<Vec<_>>();

    assert_eq!(diverging, [conservation()]);

    for (projection, present, absent) in [
        (
            explicit,
            RequiredCapability::ExactPublicAmountArithmetic,
            RequiredCapability::ConfidentialValueConservation,
        ),
        (
            private,
            RequiredCapability::ConfidentialValueConservation,
            RequiredCapability::ExactPublicAmountArithmetic,
        ),
    ] {
        let capabilities = projection.capabilities().collect::<BTreeSet<_>>();

        assert!(capabilities.contains(&present), "{:?}", projection.plan());
        assert!(!capabilities.contains(&absent), "{:?}", projection.plan());
    }
}

// --- the published censuses are the analyzed ones ---

#[test]
fn the_published_relation_census_is_the_realizations_own_live_transfer_census() {
    for scope in scopes() {
        let subject = subject(&scope);
        // One step further upstream than the analysis: the relation
        // declarations the realization authored for this operation.
        let declared = subject
            .analyzed
            .source
            .realization
            .relations
            .nodes
            .iter()
            .filter(|declaration| declaration.id.operation() == OperationId::TransferLive)
            .map(|declaration| declaration.id.clone())
            .collect::<BTreeSet<_>>();

        for projection in subject.plan.representations() {
            assert_eq!(
                projection
                    .relations()
                    .map(|requirement| requirement.relation.clone())
                    .collect::<BTreeSet<_>>(),
                declared,
                "{scope:?} {:?}",
                projection.plan(),
            );
        }
    }
}

#[test]
fn the_published_case_census_splits_every_relation_into_active_and_vacuous() {
    let subject = corruption_subject();

    for projection in subject.plan.representations() {
        let census = projection
            .relations()
            .map(|requirement| requirement.relation.clone())
            .collect::<BTreeSet<_>>();

        // Both sponsor cases, and the case identity carries this
        // projection's own representation rather than some other one.
        assert_eq!(projection.cases().count(), 2, "{:?}", projection.plan());

        for case in projection.cases() {
            assert_eq!(
                case.id.representations.get(&ObjectId::ReceiptLive),
                Some(&projection.plan().mode()),
            );

            let split = case
                .active_relations
                .union(&case.vacuous_relations)
                .cloned()
                .collect::<BTreeSet<_>>();

            assert_eq!(split, census, "{:?}", case.id);
            assert!(case.active_relations.is_disjoint(&case.vacuous_relations),);
        }
    }
}

#[test]
fn the_published_coverage_census_is_the_analyzed_relation_indexed_coverage() {
    let subject = corruption_subject();

    for projection in subject.plan.representations() {
        let analyzed = subject
            .analyzed
            .proof_plans
            .values()
            .filter(|plan| {
                plan.operations
                    .get(&OperationId::TransferLive)
                    .is_some_and(|_| {
                        plan.relation_requirements
                            .get(&conservation())
                            .is_some_and(|bundle| {
                                bundle
                                    .required_capabilities
                                    .contains(&projection.plan().conservation_capability())
                            })
                    })
            })
            .map(|plan| {
                plan.operations[&OperationId::TransferLive]
                    .coverage
                    .requirements
                    .values()
                    .map(|coverage| coverage.positive.len() + coverage.negative.len())
                    .sum::<usize>()
            })
            .collect::<BTreeSet<_>>();

        assert_eq!(
            analyzed,
            BTreeSet::from([projection.coverage().count()]),
            "{:?}",
            projection.plan(),
        );

        // Every published row is reachable by the identity it carries.
        for requirement in projection.coverage() {
            assert_eq!(
                projection.coverage_requirement(&requirement.id),
                Some(requirement),
            );
        }
    }
}

#[test]
fn equal_inputs_and_scope_permutations_produce_equal_plans() {
    // §8.3: declaration permutations produce equal projections. The
    // scope is the permutable declaration here, and the plan is a
    // property of the operation rather than of the order it was named.
    let first = subject(&[OperationId::CompactAsh, OperationId::TransferLive]).plan;
    let permuted = subject(&[OperationId::TransferLive, OperationId::CompactAsh]).plan;

    assert_eq!(first, permuted);
    assert_eq!(
        first,
        subject(&[OperationId::CompactAsh, OperationId::TransferLive]).plan,
    );
}

#[test]
fn an_out_of_scope_operation_has_no_plan() {
    let input = bound_input(&[OperationId::CompactAsh]);

    assert_eq!(
        plan_live_transfer_target_operation(&input, limits()),
        Err(CompileError::TargetOperationOutOfScope {
            operation: OperationId::TransferLive,
        }),
    );
}

#[test]
fn a_truncated_search_publishes_no_plan() {
    let input = bound_input(&[OperationId::TransferLive]);
    let truncated = PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1).expect("nonzero"),
        std::num::NonZeroU64::new(1).expect("nonzero"),
    );

    assert!(
        plan_live_transfer_target_operation(&input, truncated).is_err(),
        "a truncated search is a typed failure, never a smaller plan",
    );
}

// --- §8.3: the independent validator rejects every corruption ---

/// One admitted representation's projection inside a damaged plan.
fn damage(
    plan: &mut ValidatedLiveTransferOperationPlan,
    representation: LiveTransferRepresentationPlan,
) -> &mut crate::live_transfer_plan::LiveTransferRepresentationProjection {
    plan.representations_mut()
        .get_mut(&representation)
        .expect("an admitted representation")
}

#[test]
fn a_removed_relation_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let removed = conservation();

    damage(&mut damaged, LiveTransferRepresentationPlan::Explicit)
        .relations_mut()
        .remove(&removed);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanRelationCensusMismatch { relation: removed }),
    );
}

#[test]
fn an_added_relation_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let projection = damage(&mut damaged, LiveTransferRepresentationPlan::Explicit);
    let borrowed = projection
        .relations_mut()
        .values()
        .next()
        .cloned()
        .expect("a relation");
    let foreign = foreign_relation();

    projection.relations_mut().insert(foreign.clone(), borrowed);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanRelationCensusMismatch { relation: foreign }),
    );
}

#[test]
fn an_altered_owner_source_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();

    // The owner projection's constructibility relation is what binds
    // "every input owner" to one family; pointing it elsewhere is the
    // altered owner source §8.3 names.
    *damaged.owner_mut().constructibility_mut() = foreign_relation();

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::LiveTransferContractDefect {
            clause: LiveTransferClause::OwnerAuthorization,
        }),
    );
}

#[test]
fn an_altered_representation_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();

    damaged
        .representation_mut()
        .deferred_mut()
        .remove(&DeferredRepresentation::MixedComposition);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::LiveTransferContractDefect {
            clause: LiveTransferClause::RepresentationApproval,
        }),
    );

    // Dropping a whole admitted representation is the other half: a plan
    // for one representation is not a smaller plan for two.
    let mut dropped = subject.plan.clone();

    dropped
        .representations_mut()
        .remove(&LiveTransferRepresentationPlan::PrivateCommitted);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &dropped),
        Err(CompileError::LiveTransferContractDefect {
            clause: LiveTransferClause::RepresentationApproval,
        }),
    );
}

#[test]
fn an_altered_lifecycle_exit_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();

    damaged.lifecycle_mut().clear_outstanding();

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanLifecycleMismatch),
    );
}

#[test]
fn an_altered_carrier_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let projection = damage(
        &mut damaged,
        LiveTransferRepresentationPlan::PrivateCommitted,
    );
    let removed = projection
        .carriers_mut()
        .iter()
        .next()
        .cloned()
        .expect("a carrier row");

    projection.carriers_mut().remove(&removed);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanCarrierCensusMismatch),
    );
}

#[test]
fn an_altered_layout_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let projection = damage(&mut damaged, LiveTransferRepresentationPlan::Explicit);
    let removed = projection
        .layout_mut()
        .iter()
        .next()
        .cloned()
        .expect("a layout requirement");

    projection.layout_mut().remove(&removed);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanLayoutCensusMismatch),
    );
}

#[test]
fn an_altered_coverage_row_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let projection = damage(
        &mut damaged,
        LiveTransferRepresentationPlan::PrivateCommitted,
    );
    let removed = projection
        .coverage_mut()
        .keys()
        .next()
        .cloned()
        .expect("a coverage row");

    projection.coverage_mut().remove(&removed);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanCoverageCensusMismatch),
    );
}

#[test]
fn an_omitted_capability_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();

    damage(&mut damaged, LiveTransferRepresentationPlan::Explicit)
        .capabilities_mut()
        .remove(&RequiredCapability::ExactPublicAmountArithmetic);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanCapabilityCensusMismatch),
    );
}

#[test]
fn an_omitted_evidence_role_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();

    damage(&mut damaged, LiveTransferRepresentationPlan::Explicit)
        .external_evidence_mut()
        .clear();

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanEvidenceCensusMismatch),
    );
}

#[test]
fn an_altered_case_census_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let projection = damage(&mut damaged, LiveTransferRepresentationPlan::Explicit);
    let removed = projection
        .cases_mut()
        .keys()
        .next()
        .cloned()
        .expect("a case");

    projection.cases_mut().remove(&removed);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanCaseCensusMismatch),
    );
}

#[test]
fn a_representation_that_stopped_diverging_is_rejected() {
    // §19.4 is a two-sided claim: the conservation relation must differ
    // across the representations, and a validator that only hunted for
    // *unexpected* divergence would accept a private projection carrying
    // the explicit conservation requirement.
    //
    // Both sides are checked, at two depths. The whole validator rejects
    // this plan at its per-factor comparison, because a relation
    // requirement that is not the analyzed one fails there before any
    // cross-representation question is asked — the analysis is compared
    // first precisely so that a damaged census cannot be re-read as a
    // semantic disagreement. The cross-representation check is therefore
    // exercised directly below: it is the guard that fires if a future
    // analysis makes the two representations agree where §19.4 requires
    // them to differ, which no corruption of a plan can simulate.
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let explicit = damaged
        .projection(LiveTransferRepresentationPlan::Explicit)
        .and_then(|projection| projection.relation(&conservation()))
        .cloned()
        .expect("the explicit conservation requirement");

    damage(
        &mut damaged,
        LiveTransferRepresentationPlan::PrivateCommitted,
    )
    .relations_mut()
    .insert(conservation(), explicit);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanRelationRequirementMismatch),
    );
    assert_eq!(
        crate::live_transfer_plan::validate_representation_equivalence(&damaged),
        Err(CompileError::LiveTransferContractDefect {
            clause: LiveTransferClause::ValueConservation,
        }),
    );

    // And the undamaged plan satisfies both sides of the claim.
    assert_eq!(
        crate::live_transfer_plan::validate_representation_equivalence(&subject.plan),
        Ok(()),
    );
}

#[test]
fn a_second_diverging_relation_is_rejected() {
    // The other side of §6.6: only the conservation relation may differ,
    // so a private projection whose owner-authorization requirement
    // gained a capability the explicit one lacks is a divergence the
    // guide does not admit.
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let authorization = subject.plan.owner().authorization().clone();

    damage(
        &mut damaged,
        LiveTransferRepresentationPlan::PrivateCommitted,
    )
    .relations_mut()
    .get_mut(&authorization)
    .expect("the authorization requirement")
    .required_capabilities
    .insert(RequiredCapability::ConfidentialValueConservation);

    assert_eq!(
        crate::live_transfer_plan::validate_representation_equivalence(&damaged),
        Err(CompileError::LiveTransferRepresentationDivergence {
            relation: authorization,
        }),
    );
}

#[test]
fn an_altered_contract_projection_names_its_clause() {
    let subject = corruption_subject();

    // A family the operation admits cannot also be forbidden, and the
    // failure names the §5.4 sentence rather than the whole plan.
    let mut damaged = subject.plan.clone();

    damaged
        .class_mut()
        .forbidden_mut()
        .insert(ObjectId::ReceiptLive);

    assert_eq!(
        validate_live_transfer_plan(&subject.analyzed, &damaged),
        Err(CompileError::LiveTransferContractDefect {
            clause: LiveTransferClause::ClassClosure,
        }),
    );

    // The other contract projections answer the same way.
    for (damage, clause) in [
        (
            Box::new(|plan: &mut ValidatedLiveTransferOperationPlan| {
                plan.roots_mut().expected_mut().clear();
            }) as Box<dyn Fn(&mut ValidatedLiveTransferOperationPlan)>,
            LiveTransferClause::RootPolicy,
        ),
        (
            Box::new(|plan: &mut ValidatedLiveTransferOperationPlan| {
                plan.certificate_mut().expected_mut().clear();
            }),
            LiveTransferClause::CertificateProjection,
        ),
        (
            Box::new(|plan: &mut ValidatedLiveTransferOperationPlan| {
                plan.sponsor_mut().open_flows_mut().clear();
            }),
            LiveTransferClause::SponsorIsolation,
        ),
        (
            Box::new(|plan: &mut ValidatedLiveTransferOperationPlan| {
                plan.value_mut().flow_mut().clear();
            }),
            LiveTransferClause::ValueConservation,
        ),
    ] {
        let mut damaged = subject.plan.clone();

        damage(&mut damaged);

        assert_eq!(
            validate_live_transfer_plan(&subject.analyzed, &damaged),
            Err(CompileError::LiveTransferContractDefect { clause }),
        );
    }
}

#[test]
fn a_foreign_source_binding_is_rejected() {
    let combined = corruption_subject();
    let alone = subject(&[OperationId::TransferLive]);
    let mut damaged = combined.plan.clone();

    *damaged.source_mut() = alone.plan.source().clone();

    assert_eq!(
        validate_live_transfer_plan(&combined.analyzed, &damaged),
        Err(CompileError::TargetPlanSourceMismatch),
    );
}

#[test]
fn no_published_field_names_an_erased_sponsor_amount() {
    // §1.9 is structural here rather than textual: every published
    // layout requirement, relation-case layout requirement, and carrier
    // alternative layout, under every admitted representation, is
    // matched against the erased sponsor family by name.
    //
    // The *amount* is what is erased. Sponsor membership, cardinality,
    // isolation, and envelope multiplicity are required relations of
    // §5.8, so an authenticated census of the ordinary L-BTC family is
    // expected and is not a leak.
    for scope in scopes() {
        let plan = subject(&scope).plan;

        for projection in plan.representations() {
            for requirement in projection
                .layout()
                .chain(
                    projection
                        .relations()
                        .flat_map(|relation| relation.cases.values())
                        .flat_map(|case| case.layout_requirements.iter()),
                )
                .chain(
                    projection
                        .carriers()
                        .flat_map(|carrier| carrier.alternatives.iter())
                        .flat_map(|alternative| alternative.layout.iter()),
                )
            {
                assert!(
                    !crate::layout::names_sponsor_amount(requirement),
                    "{scope:?} {requirement:?}",
                );
            }
        }
    }
}

#[test]
fn every_published_census_is_complete_by_construction() {
    // §8.3: the `ALL` censuses are generated beside their enums, so the
    // only way one can be wrong is for the generator to be wrong. What
    // is checkable here is that each census is sorted, duplicate-free,
    // and the exact domain the plan indexes by.
    fn is_a_census<T: Copy + Ord + std::fmt::Debug>(census: &[T]) {
        assert!(
            census.windows(2).all(|pair| pair[0] < pair[1]),
            "{census:?} is not a strictly ordered census",
        );
    }

    is_a_census(LiveTransferRepresentationPlan::ALL);
    is_a_census(DeferredRepresentation::ALL);
    is_a_census(RepresentationDeferralGround::ALL);
    is_a_census(LiveTransferClause::ALL);

    let plan = corruption_subject().plan;

    assert_eq!(
        plan.representation().admitted().collect::<Vec<_>>(),
        LiveTransferRepresentationPlan::ALL,
    );
    assert_eq!(
        plan.representation()
            .deferred()
            .map(|(representation, _)| representation)
            .collect::<Vec<_>>(),
        DeferredRepresentation::ALL,
    );
    assert_eq!(
        plan.representations().count(),
        LiveTransferRepresentationPlan::ALL.len(),
    );
}

/// The census sizes this wave measured, restated as data.
///
/// A matrix derived from the plan would only prove the plan agrees with
/// itself. These are the sizes measured against the pilot realization,
/// and the plan republishes them without loss.
#[test]
fn the_published_censuses_have_the_measured_pilot_sizes() {
    let plan = corruption_subject().plan;

    for projection in plan.representations() {
        let counts: BTreeMap<&str, usize> = BTreeMap::from([
            ("cases", projection.cases().count()),
            ("coverage", projection.coverage().count()),
            ("layout", projection.layout().count()),
            ("relations", projection.relations().count()),
        ]);

        assert_eq!(
            counts,
            BTreeMap::from([
                ("cases", 2),
                ("coverage", 223),
                ("layout", 69),
                ("relations", 24),
            ]),
            "{:?}",
            projection.plan(),
        );
    }
}

/// The relation bodies the class and value projections were derived
/// from, read directly.
///
/// The one place this file reads the realization rather than the plan:
/// the projections claim to be derived from these declarations, and a
/// test that never looked at them would take that claim on trust.
#[test]
fn the_contract_projections_are_the_declared_relation_bodies() {
    let subject = corruption_subject();
    let bodies = subject
        .analyzed
        .source
        .realization
        .relations
        .nodes
        .iter()
        .filter(|declaration| declaration.id.operation() == OperationId::TransferLive)
        .map(|declaration| (declaration.id.clone(), declaration.relation.clone()))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(
        bodies.get(subject.plan.value().conservation()),
        Some(&Relation::AmountConservation {
            asset: AssetId::U,
            input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
            output_objects: BTreeSet::from([ObjectId::ReceiptLive]),
        }),
    );
    assert_eq!(
        bodies.get(subject.plan.owner().authorization()),
        Some(&Relation::OwnerAuthorization {
            object: ObjectId::ReceiptLive,
        }),
    );
    assert_eq!(
        bodies.get(subject.plan.class().closure(ObservedSide::Input)),
        Some(&Relation::AllowedObjectFamilies {
            side: ObservedSide::Input,
            allowed: BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]),
        }),
    );
    assert_eq!(
        bodies.get(subject.plan.sponsor().multiplicity()),
        Some(&Relation::SponsorEnvelopeMultiplicity {
            maximum: Count::ONE,
        }),
    );
}
