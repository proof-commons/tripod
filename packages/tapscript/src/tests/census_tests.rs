//! Census, determinism, and set-assembly tests (Guide-8 §20.5, §20.8,
//! §18.4, §18.5, Appendix G.3).

use std::collections::BTreeSet;

use compiler::target::RequiredCapability;

use super::reviewed_target;
use crate::{
    capability::{AssessmentDisposition, CapabilityAssessmentSet, assess_capability},
    error::TapscriptError,
};

#[test]
fn the_complete_census_receives_exactly_one_assessment_each() {
    let assessed = crate::assess_complete_census(&reviewed_target())
        .expect("the complete census assesses without disagreeing with itself");

    let keys: Vec<_> = assessed
        .assessments()
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
            .assessment(*capability)
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
        .projection()
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
fn assessment_is_deterministic_and_input_order_free() {
    let target = reviewed_target();

    assert_eq!(
        crate::assess_complete_census(&target).expect("census"),
        crate::assess_complete_census(&target).expect("census"),
        "repeated assessment of one target is equal",
    );

    // A permuted census is the same census: the input is a set and the
    // result is keyed by the compiler's own order, so no caller's
    // iteration order can reach the projection.
    let forward: BTreeSet<_> = RequiredCapability::ALL.iter().copied().collect();
    let reversed: BTreeSet<_> = RequiredCapability::ALL.iter().rev().copied().collect();

    let assessed_forward = forward
        .iter()
        .map(|capability| (*capability, assess_capability(&target, *capability)))
        .collect();
    let assessed_reversed = reversed
        .iter()
        .rev()
        .map(|capability| (*capability, assess_capability(&target, *capability)))
        .collect();

    assert_eq!(
        CapabilityAssessmentSet::assemble(&forward, assessed_forward).expect("forward"),
        CapabilityAssessmentSet::assemble(&reversed, assessed_reversed).expect("reversed"),
    );
}

#[test]
fn assessing_one_capability_twice_is_rejected() {
    let target = reviewed_target();
    let required = BTreeSet::from([RequiredCapability::PublicConstructibility]);
    let repeated = vec![
        (
            RequiredCapability::PublicConstructibility,
            assess_capability(&target, RequiredCapability::PublicConstructibility),
        ),
        (
            RequiredCapability::PublicConstructibility,
            assess_capability(&target, RequiredCapability::PublicConstructibility),
        ),
    ];

    assert_eq!(
        CapabilityAssessmentSet::assemble(&required, repeated).unwrap_err(),
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
        assess_capability(&target, RequiredCapability::OwnerAuthorization),
    )];

    assert_eq!(
        CapabilityAssessmentSet::assemble(&required, partial).unwrap_err(),
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
            assess_capability(&target, RequiredCapability::OwnerAuthorization),
        ),
        (
            RequiredCapability::AuthenticatedRootEffects,
            assess_capability(&target, RequiredCapability::AuthenticatedRootEffects),
        ),
    ];

    assert_eq!(
        CapabilityAssessmentSet::assemble(&required, surplus).unwrap_err(),
        TapscriptError::CapabilityAssessmentCensusMismatch {
            missing: vec![],
            unexpected: vec![RequiredCapability::AuthenticatedRootEffects],
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
    assert!(mismatch.to_string().contains("1 missing, 0 unexpected"));
}

#[test]
fn no_assessment_claims_a_complete_backend_pattern() {
    // Guide-8 §16.5. The claim is structural rather than statistical:
    // `BackendPatternId` is uninhabited, so the variant has no value at
    // all. This walks the census anyway, because a reader checking the
    // prohibition should be able to see it checked.
    let assessed = crate::assess_complete_census(&reviewed_target()).expect("census");

    assert!(assessed.projection().iter().all(
        |projection| projection.disposition() != AssessmentDisposition::CompleteBackendPattern
    ),);
}
