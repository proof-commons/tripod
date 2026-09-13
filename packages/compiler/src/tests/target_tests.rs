//! Abstract target requirement boundary tests (Guide-8 §15, §20.4,
//! Appendix G.2).
//!
//! The positive assertions are checked against something derived
//! independently of the projection: the capability census against the
//! union the analyzed program's own relations own, and the evidence
//! roles against the realization requirements the analysis retained.
//! The negative assertions corrupt an assembled program or a census
//! constant and require the projection to refuse rather than publish a
//! set that no longer describes an analysis.

use std::collections::BTreeSet;

use architecture::OperationId;
use realization::ExternalEvidenceRequirement;

use super::bound_input;
use crate::{
    CompileError,
    analyzed::{ScopedAnalyzedProgram, analyze_scoped_program},
    capability::RequiredCapability,
    placement::PlacementSearchLimits,
    target::{
        ExternalEvidenceRole, TargetRequirementSet, analyze_target_requirements,
        project_target_requirements,
    },
};

/// Generous limits: a truncated pilot search would hide a defect
/// rather than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

fn pilot_scope() -> [OperationId; 2] {
    [OperationId::CompactAsh, OperationId::TransferLive]
}

fn analyzed(operations: &[OperationId]) -> ScopedAnalyzedProgram {
    analyze_scoped_program(&bound_input(operations), limits()).expect("pilot analysis")
}

fn requirements(operations: &[OperationId]) -> TargetRequirementSet {
    analyze_target_requirements(&bound_input(operations), limits()).expect("pilot requirements")
}

// --- the census constant is the type's own order, exactly ---

#[test]
fn the_capability_census_is_complete_and_duplicate_free() {
    // An independently written expectation, not a fold over `ALL`: a
    // census compared only with itself agrees with itself. Since
    // `census_enum!` generates `ALL` from the enum, this literal no
    // longer guards the two against each other — that drift is now
    // unwriteable. It pins the membership itself, so adding, removing,
    // or reordering a capability is a visible test change and not a
    // silent one.
    let expected = [
        RequiredCapability::AuthenticatedObjectRecognition,
        RequiredCapability::AuthenticatedFamilyCardinality,
        RequiredCapability::AuthenticatedCanonicalPartition,
        RequiredCapability::AuthenticatedOpenFlowPartition,
        RequiredCapability::AuthenticatedRootEffects,
        RequiredCapability::AuthenticatedProjectionSet,
        RequiredCapability::ExactPublicAmountArithmetic,
        RequiredCapability::ConfidentialValueConservation,
        RequiredCapability::OwnerAuthorization,
        RequiredCapability::OperatorAuthorization,
        RequiredCapability::RefundAuthorization,
        RequiredCapability::PublicConstructibility,
        RequiredCapability::WholeTransactionValueConservation,
    ];

    assert_eq!(RequiredCapability::ALL, expected);
    assert_eq!(
        RequiredCapability::ALL
            .iter()
            .collect::<BTreeSet<_>>()
            .len(),
        RequiredCapability::ALL.len(),
        "a census that names one capability twice is not a census",
    );
    assert!(
        RequiredCapability::ALL
            .windows(2)
            .all(|pair| pair[0] < pair[1]),
        "census order is the type's own canonical order",
    );
}

#[test]
fn the_evidence_role_census_is_complete_and_duplicate_free() {
    assert_eq!(
        ExternalEvidenceRole::ALL,
        [
            ExternalEvidenceRole::ConfidentialValueConservation,
            ExternalEvidenceRole::SubstrateConservation,
            ExternalEvidenceRole::OperatorAuthorization,
        ],
    );
    assert!(
        ExternalEvidenceRole::ALL
            .windows(2)
            .all(|pair| pair[0] < pair[1]),
    );
}

#[test]
fn every_realization_requirement_class_projects_to_a_role() {
    // The mapping is total by construction — the match has no wildcard
    // arm — and this fixture states the one class that exists today so
    // a class added later is a visible test change as well as a
    // compile failure.
    let retained = analyzed(&[OperationId::TransferLive]).required_external_evidence;
    let classes = retained
        .iter()
        .map(|requirement| match requirement {
            ExternalEvidenceRequirement::ConfidentialValueConservation { .. } => {
                ExternalEvidenceRole::ConfidentialValueConservation
            }
            ExternalEvidenceRequirement::OperatorAuthorization { .. } => {
                ExternalEvidenceRole::OperatorAuthorization
            }
            ExternalEvidenceRequirement::SubstrateConservation { .. } => {
                ExternalEvidenceRole::SubstrateConservation
            }
        })
        .collect::<BTreeSet<_>>();

    // Both classes, stated as a set rather than as "not empty": a live
    // transfer leaves the substrate obligation open under either
    // representation and the confidential one open under the private
    // representation, so an analysis that lost the second would still
    // satisfy a non-emptiness check.
    assert_eq!(
        classes,
        BTreeSet::from([
            ExternalEvidenceRole::ConfidentialValueConservation,
            ExternalEvidenceRole::SubstrateConservation,
        ]),
    );
}

// --- the projection describes the analysis it came from ---

#[test]
fn the_projection_is_the_union_of_the_relation_owned_capabilities() {
    let program = analyzed(&pilot_scope());

    // Derived from the relations rather than from the aggregates the
    // projection also reads.
    let expected = program
        .proof_plans
        .values()
        .flat_map(|plan| plan.relation_requirements.values())
        .flat_map(|bundle| bundle.required_capabilities.iter().copied())
        .collect::<BTreeSet<_>>();

    let projected = requirements(&pilot_scope())
        .capabilities()
        .collect::<BTreeSet<_>>();

    assert_eq!(projected, expected);
    assert!(
        !expected.is_empty(),
        "the pilot scope requires capabilities; an empty expectation would pass vacuously",
    );
}

#[test]
fn the_projection_carries_every_open_evidence_role() {
    let program = analyzed(&pilot_scope());
    let expected = program
        .required_external_evidence
        .iter()
        .map(|requirement| match requirement {
            ExternalEvidenceRequirement::ConfidentialValueConservation { .. } => {
                ExternalEvidenceRole::ConfidentialValueConservation
            }
            ExternalEvidenceRequirement::OperatorAuthorization { .. } => {
                ExternalEvidenceRole::OperatorAuthorization
            }
            ExternalEvidenceRequirement::SubstrateConservation { .. } => {
                ExternalEvidenceRole::SubstrateConservation
            }
        })
        .collect::<BTreeSet<_>>();

    let projected = requirements(&pilot_scope())
        .external_evidence()
        .collect::<BTreeSet<_>>();

    assert_eq!(projected, expected);
    assert!(!expected.is_empty(), "the pilots leave evidence open");
}

#[test]
fn the_projection_is_published_in_canonical_census_order() {
    let projected = requirements(&pilot_scope())
        .capabilities()
        .collect::<Vec<_>>();

    assert!(
        projected.windows(2).all(|pair| pair[0] < pair[1]),
        "iteration is strictly ascending, so no member is published twice",
    );

    let census = RequiredCapability::ALL
        .iter()
        .copied()
        .filter(|capability| projected.contains(capability))
        .collect::<Vec<_>>();

    assert_eq!(projected, census);
}

#[test]
fn repeated_and_permuted_analysis_projects_equal_requirements() {
    assert_eq!(requirements(&pilot_scope()), requirements(&pilot_scope()));
    assert_eq!(
        requirements(&[OperationId::CompactAsh, OperationId::TransferLive]),
        requirements(&[OperationId::TransferLive, OperationId::CompactAsh]),
    );
}

#[test]
fn a_narrower_scope_never_gains_a_requirement_the_wider_one_lacks() {
    let wide = requirements(&pilot_scope())
        .capabilities()
        .collect::<BTreeSet<_>>();

    for operation in pilot_scope() {
        let narrow = requirements(&[operation])
            .capabilities()
            .collect::<BTreeSet<_>>();

        assert!(
            narrow.is_subset(&wide),
            "the union over a subset of operations is contained in the union over all of them",
        );
    }
}

// --- G.2: a corrupted analysis does not reach the boundary ---

#[test]
fn an_aggregate_capability_no_relation_owns_is_rejected() {
    let mut program = analyzed(&[OperationId::TransferLive]);
    let plan = program
        .proof_plans
        .values_mut()
        .next()
        .expect("the pilot analysis retains a plan");

    // A capability this plan does not already require, so the mutation
    // is a real difference rather than a silent no-op.
    let unowned = *RequiredCapability::ALL
        .iter()
        .find(|capability| !plan.proof_plan.required_capabilities.contains(capability))
        .expect("the pilot plan does not require every capability");

    plan.proof_plan.required_capabilities.insert(unowned);

    assert_eq!(
        project_target_requirements(&program).unwrap_err(),
        CompileError::AnalyzedCapabilityClosureMismatch {
            relation: None,
            capability: unowned,
        },
    );
}

#[test]
fn a_relation_owned_capability_the_aggregate_dropped_is_rejected() {
    let mut program = analyzed(&[OperationId::TransferLive]);
    let plan = program
        .proof_plans
        .values_mut()
        .next()
        .expect("the pilot analysis retains a plan");

    let (relation, capability) = plan
        .relation_requirements
        .iter()
        .find_map(|(relation, bundle)| {
            bundle
                .required_capabilities
                .first()
                .map(|capability| (relation.clone(), *capability))
        })
        .expect("some relation owns some capability");

    plan.proof_plan.required_capabilities.remove(&capability);

    assert_eq!(
        project_target_requirements(&program).unwrap_err(),
        CompileError::AnalyzedCapabilityClosureMismatch {
            relation: Some(relation),
            capability,
        },
    );
}

#[test]
fn an_evidence_requirement_of_an_uncensused_role_is_unrepresentable() {
    // Stated rather than tested by mutation: the role projection is a
    // total function over the realization's requirement classes, so
    // there is no value of that type whose role is absent from the
    // census. The corresponding failure is reachable only through the
    // census constant itself, which the next tests corrupt directly.
    let roles = ExternalEvidenceRole::ALL.iter().collect::<BTreeSet<_>>();

    assert!(roles.contains(&ExternalEvidenceRole::SubstrateConservation));
}

// --- G.2: a corrupted census constant does not reach the boundary ---

#[test]
fn a_repeated_census_member_is_rejected() {
    let present = BTreeSet::from([RequiredCapability::OwnerAuthorization]);
    let census = [
        RequiredCapability::OwnerAuthorization,
        RequiredCapability::OwnerAuthorization,
    ];

    assert_eq!(
        crate::target::canonical_census(&present, &census, |capability| {
            CompileError::NoncanonicalCapabilityCensus { capability }
        })
        .unwrap_err(),
        CompileError::NoncanonicalCapabilityCensus {
            capability: RequiredCapability::OwnerAuthorization,
        },
    );
}

#[test]
fn a_misordered_census_is_rejected() {
    let present = BTreeSet::new();
    let census = [
        RequiredCapability::OwnerAuthorization,
        RequiredCapability::AuthenticatedObjectRecognition,
    ];

    assert_eq!(
        crate::target::canonical_census(&present, &census, |capability| {
            CompileError::NoncanonicalCapabilityCensus { capability }
        })
        .unwrap_err(),
        CompileError::NoncanonicalCapabilityCensus {
            capability: RequiredCapability::AuthenticatedObjectRecognition,
        },
    );
}

#[test]
fn a_required_capability_the_census_omits_is_rejected() {
    let present = [
        RequiredCapability::OwnerAuthorization,
        RequiredCapability::PublicConstructibility,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let census = [RequiredCapability::OwnerAuthorization];

    assert_eq!(
        crate::target::canonical_census(&present, &census, |capability| {
            CompileError::NoncanonicalCapabilityCensus { capability }
        })
        .unwrap_err(),
        CompileError::NoncanonicalCapabilityCensus {
            capability: RequiredCapability::PublicConstructibility,
        },
    );
}

#[test]
fn an_evidence_role_the_census_omits_is_rejected() {
    let present = BTreeSet::from([ExternalEvidenceRole::SubstrateConservation]);

    assert_eq!(
        crate::target::canonical_census(&present, &[], |role| {
            CompileError::NoncanonicalEvidenceRoleCensus { role }
        })
        .unwrap_err(),
        CompileError::NoncanonicalEvidenceRoleCensus {
            role: ExternalEvidenceRole::SubstrateConservation,
        },
    );
}

#[test]
fn the_complete_census_orders_the_present_members() {
    let present = [
        RequiredCapability::PublicConstructibility,
        RequiredCapability::AuthenticatedObjectRecognition,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    assert_eq!(
        crate::target::canonical_census(&present, RequiredCapability::ALL, |capability| {
            CompileError::NoncanonicalCapabilityCensus { capability }
        })
        .expect("a complete census accepts its own members"),
        vec![
            RequiredCapability::AuthenticatedObjectRecognition,
            RequiredCapability::PublicConstructibility,
        ],
    );
}

#[test]
fn operator_evidence_projects_to_its_own_role() {
    assert_eq!(
        ExternalEvidenceRole::of(&ExternalEvidenceRequirement::OperatorAuthorization {
            operation: OperationId::AnnounceMaturity
        }),
        ExternalEvidenceRole::OperatorAuthorization
    );
}
