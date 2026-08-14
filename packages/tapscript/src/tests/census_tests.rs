//! Census, determinism, and set-assembly tests (Guide-8 §20.5, §20.8,
//! §18.4, §18.5, Appendix G.3).

use std::collections::BTreeSet;

use compiler::target::{ExternalEvidenceRole, RequiredCapability};

use super::reviewed_target;
use crate::{
    capability::{
        AssessmentDisposition, TargetAssessmentSet, assess_evidence_role, assess_static_capability,
    },
    error::TapscriptError,
};

/// The complete compiler capability census, as a set.
fn every_capability() -> BTreeSet<RequiredCapability> {
    RequiredCapability::ALL.iter().copied().collect()
}

/// The complete compiler evidence-role census, as a set.
fn every_role() -> BTreeSet<ExternalEvidenceRole> {
    ExternalEvidenceRole::ALL.iter().copied().collect()
}

/// One assessment for each member of the complete capability census.
fn assess_every_capability(
    target: &target_elements::ReviewedElementsTapscriptDefinition,
) -> Vec<(
    RequiredCapability,
    crate::capability::StaticCapabilityAssessment,
)> {
    RequiredCapability::ALL
        .iter()
        .map(|capability| (*capability, assess_static_capability(target, *capability)))
        .collect()
}

/// One assessment for each member of the complete role census.
fn assess_every_role() -> Vec<(
    ExternalEvidenceRole,
    crate::capability::ExternalEvidenceAssessment,
)> {
    ExternalEvidenceRole::ALL
        .iter()
        .map(|role| (*role, assess_evidence_role(*role)))
        .collect()
}

#[test]
fn the_complete_census_receives_exactly_one_assessment_each() {
    let assessed = crate::assess_complete_census(&reviewed_target())
        .expect("the complete census assesses without disagreeing with itself");

    let keys: Vec<_> = assessed
        .capability_assessments()
        .map(|(capability, _)| capability)
        .collect();

    assert_eq!(keys, RequiredCapability::ALL);
    assert_eq!(
        keys.iter().collect::<BTreeSet<_>>().len(),
        keys.len(),
        "no capability is assessed twice",
    );

    for capability in RequiredCapability::ALL {
        let assessment = assessed
            .capability_assessment(*capability)
            .expect("every censused capability has an assessment");
        assert_eq!(
            assessment.required(),
            *capability,
            "an assessment answers the capability it is filed under",
        );
    }
}

#[test]
fn the_set_projection_is_the_census_in_order() {
    let assessed = crate::assess_complete_census(&reviewed_target()).expect("census");
    let projected: Vec<_> = assessed
        .capability_projection()
        .into_iter()
        .map(|projection| projection.required())
        .collect();

    assert_eq!(projected, RequiredCapability::ALL);
    assert!(
        projected.windows(2).all(|pair| pair[0] < pair[1]),
        "the vector projection ascends strictly, so no member is published twice",
    );
}

#[test]
fn the_evidence_role_census_receives_exactly_one_assessment_each() {
    let assessed = crate::assess_complete_census(&reviewed_target())
        .expect("the complete censuses assess without disagreeing with themselves");

    let keys: Vec<_> = assessed
        .evidence_assessments()
        .map(|(role, _)| role)
        .collect();

    assert_eq!(keys, ExternalEvidenceRole::ALL);
    assert_eq!(
        keys.iter().collect::<BTreeSet<_>>().len(),
        keys.len(),
        "no evidence role is assessed twice",
    );

    for role in ExternalEvidenceRole::ALL {
        let assessment = assessed
            .evidence_assessment(*role)
            .expect("every censused role has an assessment");
        assert_eq!(
            assessment.role(),
            *role,
            "an assessment answers the role it is filed under",
        );
    }
}

#[test]
fn the_evidence_projection_is_the_census_in_order() {
    let assessed = crate::assess_complete_census(&reviewed_target()).expect("census");
    let projected: Vec<_> = assessed
        .evidence_projection()
        .into_iter()
        .map(|projection| projection.role())
        .collect();

    assert_eq!(projected, ExternalEvidenceRole::ALL);
    assert!(
        projected.windows(2).all(|pair| pair[0] < pair[1]),
        "the vector projection ascends strictly, so no role is published twice",
    );
}

#[test]
fn assessment_is_deterministic_and_input_order_free() {
    let target = reviewed_target();

    assert_eq!(
        crate::assess_complete_census(&target).expect("census"),
        crate::assess_complete_census(&target).expect("census"),
        "repeated assessment of one target is equal",
    );

    // A permuted census is the same census: the input is a set and the
    // result is keyed by the compiler's own order, so no caller's
    // iteration order can reach either projection. Both censuses are
    // permuted here, because either one could have carried an
    // order-dependent assembly.
    let forward: BTreeSet<_> = RequiredCapability::ALL.iter().copied().collect();
    let reversed: BTreeSet<_> = RequiredCapability::ALL.iter().rev().copied().collect();
    let roles_forward: BTreeSet<_> = ExternalEvidenceRole::ALL.iter().copied().collect();
    let roles_reversed: BTreeSet<_> = ExternalEvidenceRole::ALL.iter().rev().copied().collect();

    let assessed_forward = forward
        .iter()
        .map(|capability| (*capability, assess_static_capability(&target, *capability)))
        .collect();
    let assessed_reversed = reversed
        .iter()
        .rev()
        .map(|capability| (*capability, assess_static_capability(&target, *capability)))
        .collect();
    let evidence_forward = roles_forward
        .iter()
        .map(|role| (*role, assess_evidence_role(*role)))
        .collect();
    let evidence_reversed = roles_reversed
        .iter()
        .rev()
        .map(|role| (*role, assess_evidence_role(*role)))
        .collect();

    assert_eq!(
        TargetAssessmentSet::assemble(&forward, assessed_forward, &roles_forward, evidence_forward)
            .expect("forward"),
        TargetAssessmentSet::assemble(
            &reversed,
            assessed_reversed,
            &roles_reversed,
            evidence_reversed,
        )
        .expect("reversed"),
    );
}

#[test]
fn assessing_one_capability_twice_is_rejected() {
    let target = reviewed_target();
    let required = BTreeSet::from([RequiredCapability::PublicConstructibility]);
    let repeated = vec![
        (
            RequiredCapability::PublicConstructibility,
            assess_static_capability(&target, RequiredCapability::PublicConstructibility),
        ),
        (
            RequiredCapability::PublicConstructibility,
            assess_static_capability(&target, RequiredCapability::PublicConstructibility),
        ),
    ];

    assert_eq!(
        TargetAssessmentSet::assemble(&required, repeated, &every_role(), assess_every_role())
            .unwrap_err(),
        TapscriptError::DuplicateCapabilityAssessment(RequiredCapability::PublicConstructibility),
    );
}

#[test]
fn an_omitted_assessment_is_rejected() {
    let target = reviewed_target();
    let required = BTreeSet::from([
        RequiredCapability::PublicConstructibility,
        RequiredCapability::OwnerAuthorization,
    ]);
    let partial = vec![(
        RequiredCapability::OwnerAuthorization,
        assess_static_capability(&target, RequiredCapability::OwnerAuthorization),
    )];

    assert_eq!(
        TargetAssessmentSet::assemble(&required, partial, &every_role(), assess_every_role())
            .unwrap_err(),
        TapscriptError::CapabilityAssessmentCensusMismatch {
            missing: vec![RequiredCapability::PublicConstructibility],
            unexpected: vec![],
        },
    );
}

#[test]
fn an_assessment_nothing_required_is_rejected() {
    let target = reviewed_target();
    let required = BTreeSet::from([RequiredCapability::OwnerAuthorization]);
    let surplus = vec![
        (
            RequiredCapability::OwnerAuthorization,
            assess_static_capability(&target, RequiredCapability::OwnerAuthorization),
        ),
        (
            RequiredCapability::AuthenticatedRootEffects,
            assess_static_capability(&target, RequiredCapability::AuthenticatedRootEffects),
        ),
    ];

    assert_eq!(
        TargetAssessmentSet::assemble(&required, surplus, &every_role(), assess_every_role())
            .unwrap_err(),
        TapscriptError::CapabilityAssessmentCensusMismatch {
            missing: vec![],
            unexpected: vec![RequiredCapability::AuthenticatedRootEffects],
        },
    );
}

#[test]
fn assessing_one_evidence_role_twice_is_rejected() {
    let required = BTreeSet::from([ExternalEvidenceRole::SubstrateConservation]);
    let repeated = vec![
        (
            ExternalEvidenceRole::SubstrateConservation,
            assess_evidence_role(ExternalEvidenceRole::SubstrateConservation),
        ),
        (
            ExternalEvidenceRole::SubstrateConservation,
            assess_evidence_role(ExternalEvidenceRole::SubstrateConservation),
        ),
    ];

    assert_eq!(
        TargetAssessmentSet::assemble(&BTreeSet::new(), vec![], &required, repeated).unwrap_err(),
        TapscriptError::DuplicateEvidenceAssessment(ExternalEvidenceRole::SubstrateConservation),
    );
}

#[test]
fn an_omitted_evidence_assessment_is_rejected() {
    // The exact failure R2-C05 describes: a compiler evidence role that
    // reaches the adapter and stops there.
    let required = BTreeSet::from([ExternalEvidenceRole::SubstrateConservation]);

    assert_eq!(
        TargetAssessmentSet::assemble(&BTreeSet::new(), vec![], &required, vec![]).unwrap_err(),
        TapscriptError::EvidenceAssessmentCensusMismatch {
            missing: vec![ExternalEvidenceRole::SubstrateConservation],
            unexpected: vec![],
        },
    );
}

#[test]
fn an_evidence_assessment_nothing_required_is_rejected() {
    let surplus = vec![(
        ExternalEvidenceRole::SubstrateConservation,
        assess_evidence_role(ExternalEvidenceRole::SubstrateConservation),
    )];

    assert_eq!(
        TargetAssessmentSet::assemble(&BTreeSet::new(), vec![], &BTreeSet::new(), surplus)
            .unwrap_err(),
        TapscriptError::EvidenceAssessmentCensusMismatch {
            missing: vec![],
            unexpected: vec![ExternalEvidenceRole::SubstrateConservation],
        },
    );
}

#[test]
fn both_censuses_are_assembled_from_the_same_call() {
    // Neither half may be optional. Assembling the complete capability
    // census with no roles is a rejection, and the mirror image is too,
    // so no caller can obtain a set that answers one published census
    // and silently drops the other.
    let target = reviewed_target();

    assert_eq!(
        TargetAssessmentSet::assemble(
            &every_capability(),
            assess_every_capability(&target),
            &every_role(),
            vec![],
        )
        .unwrap_err(),
        TapscriptError::EvidenceAssessmentCensusMismatch {
            missing: ExternalEvidenceRole::ALL.to_vec(),
            unexpected: vec![],
        },
    );

    assert_eq!(
        TargetAssessmentSet::assemble(
            &every_capability(),
            vec![],
            &every_role(),
            assess_every_role(),
        )
        .unwrap_err(),
        TapscriptError::CapabilityAssessmentCensusMismatch {
            missing: RequiredCapability::ALL.to_vec(),
            unexpected: vec![],
        },
    );
}

#[test]
fn the_error_root_displays_and_is_a_standard_error() {
    fn assert_error<E: std::error::Error>(_: &E) {}

    let duplicate =
        TapscriptError::DuplicateCapabilityAssessment(RequiredCapability::OwnerAuthorization);
    assert!(duplicate.to_string().contains("assessed twice"));
    assert_error(&duplicate);

    let mismatch = TapscriptError::CapabilityAssessmentCensusMismatch {
        missing: vec![RequiredCapability::OwnerAuthorization],
        unexpected: vec![],
    };
    assert!(
        mismatch
            .to_string()
            .contains("capability assessment census")
    );
    assert!(mismatch.to_string().contains("1 missing, 0 unexpected"));

    let duplicate_role =
        TapscriptError::DuplicateEvidenceAssessment(ExternalEvidenceRole::SubstrateConservation);
    assert!(duplicate_role.to_string().contains("assessed twice"));
    assert_error(&duplicate_role);

    let role_mismatch = TapscriptError::EvidenceAssessmentCensusMismatch {
        missing: vec![],
        unexpected: vec![ExternalEvidenceRole::SubstrateConservation],
    };
    assert!(
        role_mismatch
            .to_string()
            .contains("evidence assessment census")
    );
    assert!(
        role_mismatch
            .to_string()
            .contains("0 missing, 1 unexpected")
    );
}

#[test]
fn no_assessment_claims_a_complete_backend_pattern() {
    // Guide-8 §16.5. The claim is structural rather than statistical:
    // `BackendPatternId` is uninhabited, so the variant has no value at
    // all. This walks the census anyway, because a reader checking the
    // prohibition should be able to see it checked.
    let assessed = crate::assess_complete_census(&reviewed_target()).expect("census");

    assert!(assessed.capability_projection().iter().all(
        |projection| projection.disposition() != AssessmentDisposition::CompleteBackendPattern
    ),);
}
