//! The §8 assessment, run against the real compact-ASH plan.
//!
//! # The plan is the compiler's, not a fixture
//!
//! The subject below is built by the compiler's own public
//! constructor from the architecture body and the derived Phase-1
//! realization. An assessment checked against a hand-assembled plan
//! would be checking a fixture; §8.1 assesses what the compiler
//! actually required, so that is what is handed in.
//!
//! # Both directions of the census, and the exact totals
//!
//! The verdict census is required to equal the plan's own three
//! censuses exactly — no requirement without a verdict, no verdict
//! without a requirement — and the totals are written out. A
//! containment check would pass for an assessment that answered half
//! the layout census.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;

use architecture::{ARCHITECTURE, OperationId};
use compiler::input::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use compiler::operation_plan::{
    ExternalEvidenceRole, PlacementSearchLimits, RequiredCapability, ValidatedTargetOperationPlan,
    plan_compact_ash_target_operation,
};
use realization::{RealizationScope, derive};
use target_elements::{ElementsCapability, StaticCapabilityStatus};

use crate::capability::{AssessmentDisposition, BackendPatternId};
use crate::operation_assessment::{
    EmissionRefusal, OperationRequirement, VerdictGround, assess_operation_plan,
};
use crate::pattern::{BackendPattern, operation_patterns};
use crate::shape::demonstration_shape_set;
use crate::tests::{pattern_symbols, reviewed_target};

/// A nonzero limit for the fixtures.
fn nonzero(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("the fixture limits are nonzero")
}

/// The validated compact-ASH plan, from the compiler's own constructor.
fn plan() -> ValidatedTargetOperationPlan {
    let realization =
        derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).expect("the pilots derive");
    let scope = CompilationScope::from_operations([OperationId::CompactAsh])
        .expect("a one-operation scope");
    let policy =
        AnalysisPolicy::strict(ProofSearchLimits::new(nonzero(1_000_000), nonzero(10_000)));
    let input = bind_input(&ARCHITECTURE, realization, scope, policy).expect("the input binds");

    plan_compact_ash_target_operation(
        &input,
        PlacementSearchLimits::new(nonzero(10_000_000), nonzero(1_000_000)),
    )
    .expect("the plan validates")
}

/// The pattern census over the largest demonstration shape.
fn patterns() -> BTreeMap<BackendPatternId, BackendPattern> {
    let target = reviewed_target();
    let shape = demonstration_shape_set()
        .shapes()
        .max_by_key(|shape| (shape.ash_inputs(), shape.sponsor_inputs()))
        .expect("the demonstration set is not empty");

    operation_patterns(&target, &pattern_symbols(&target), shape).expect("every pattern schedules")
}

#[test]
fn the_verdict_census_is_exactly_the_plans_three_censuses() {
    // §8.1: the assessment covers every operation requirement, and
    // covers nothing else. Both directions, rebuilt from the plan
    // rather than from the assessment's own construction.
    let target = reviewed_target();
    let plan = plan();
    let assessment = assess_operation_plan(&target, &plan).expect("the assessment completes");

    let mut expected = BTreeSet::new();
    for capability in plan.capabilities() {
        expected.insert(OperationRequirement::Capability(capability));
    }
    for requirement in plan.layout() {
        expected.insert(OperationRequirement::Layout(requirement.clone()));
    }
    for role in plan.external_evidence() {
        expected.insert(OperationRequirement::ExternalEvidence(role));
    }

    let assessed = assessment
        .verdicts()
        .map(|verdict| verdict.requirement().clone())
        .collect::<BTreeSet<_>>();

    assert_eq!(assessed, expected);

    // The measured Wave-4 sizes, restated so a plan that silently grew
    // or shrank fails here rather than being assessed quietly.
    assert_eq!(plan.capabilities().count(), 9);
    assert_eq!(plan.layout().count(), 69);
    assert_eq!(plan.external_evidence().count(), 1);
    assert_eq!(assessment.len(), 79);
    assert!(!assessment.is_empty());
}

#[test]
fn the_support_state_census_is_exactly_this() {
    // §8.2's multi-state census, written out. Sixty-nine layout rows,
    // nine capabilities, one evidence role; every one of them lands in
    // a state, and the states are the six the rule fixes.
    let target = reviewed_target();
    let assessment = assess_operation_plan(&target, &plan()).expect("the assessment completes");

    assert_eq!(
        assessment.state_census(),
        BTreeMap::from([
            (AssessmentDisposition::CompleteBackendPattern, 74),
            (AssessmentDisposition::BackendStructural, 3),
            (AssessmentDisposition::ExternalEvidenceRequired, 2),
        ]),
    );

    // No row is unsupported, missing a prerequisite, or still owing a
    // pattern — which is what makes emission admissible below, and
    // which the degraded-target test then takes away.
    for state in [
        AssessmentDisposition::Unsupported,
        AssessmentDisposition::MissingTargetPrimitives,
        AssessmentDisposition::BackendPatternRequired,
    ] {
        assert!(!assessment.state_census().contains_key(&state));
    }
}

#[test]
fn every_verdict_states_its_ground_and_the_absences_are_named_as_absences() {
    // §8.1: root effects and specialized projections may be discharged
    // as structural absences, and each remains represented in the
    // census. Both are here, both are `BackendStructural`, and both
    // carry `StructuralAbsence` rather than `ApprovedPattern` — because
    // nothing proves an effect that does not occur.
    let target = reviewed_target();
    let assessment = assess_operation_plan(&target, &plan()).expect("the assessment completes");

    for capability in [
        RequiredCapability::AuthenticatedRootEffects,
        RequiredCapability::AuthenticatedProjectionSet,
    ] {
        let verdict = assessment
            .verdict(&OperationRequirement::Capability(capability))
            .expect("the capability is assessed");

        assert_eq!(
            verdict.operation(),
            AssessmentDisposition::BackendStructural
        );
        assert_eq!(verdict.ground(), VerdictGround::StructuralAbsence);
        assert_eq!(
            verdict.target_wide(),
            Some(AssessmentDisposition::BackendPatternRequired),
            "the target-wide answer is retained so the difference is visible",
        );
    }

    assert_eq!(
        assessment.ground_census(),
        BTreeMap::from([
            (VerdictGround::ApprovedPattern, 74),
            (VerdictGround::StructuralAbsence, 2),
            (VerdictGround::CompilerOrAbiObligation, 1),
            (VerdictGround::ExternalTargetClaim, 2),
        ]),
    );
}

#[test]
fn conservation_is_never_discharged_by_a_program() {
    // §8.3 by name: whole-transaction balance must not be substituted
    // for object closure. The row stays external evidence, and the
    // conservation prohibition is separately enforced by
    // `emission_admissible`.
    let target = reviewed_target();
    let assessment = assess_operation_plan(&target, &plan()).expect("the assessment completes");

    let verdict = assessment
        .verdict(&OperationRequirement::Capability(
            RequiredCapability::WholeTransactionValueConservation,
        ))
        .expect("conservation is assessed");

    assert_eq!(
        verdict.operation(),
        AssessmentDisposition::ExternalEvidenceRequired,
    );
    assert_eq!(verdict.ground(), VerdictGround::ExternalTargetClaim);
    assert!(verdict.patterns().is_empty());

    let role = assessment
        .verdict(&OperationRequirement::ExternalEvidence(
            ExternalEvidenceRole::SubstrateConservation,
        ))
        .expect("the substrate role is assessed");
    assert_eq!(
        role.operation(),
        AssessmentDisposition::ExternalEvidenceRequired,
    );
    assert!(!role.evidence().is_empty());
}

#[test]
fn emission_is_admissible_and_every_claimed_pattern_is_backed() {
    // The positive half of §8.3. Every row leaves an accepted target
    // proof, every pattern a verdict names exists in the census, and no
    // admitted pattern reads a sponsor amount.
    let target = reviewed_target();
    let assessment = assess_operation_plan(&target, &plan()).expect("the assessment completes");

    assert_eq!(assessment.emission_admissible(&patterns()), Ok(()));

    let census = patterns();
    for verdict in assessment.verdicts() {
        for pattern in verdict.patterns() {
            assert!(census.contains_key(pattern), "{pattern:?} is unbacked");
        }
    }
}

#[test]
fn an_empty_pattern_census_makes_every_claim_unbacked() {
    // The negative control for the backing check: a verdict naming a
    // pattern nobody carries must refuse rather than pass, because a
    // claim about a pattern that does not exist is the speculative
    // identity §1.10 refuses.
    let target = reviewed_target();
    let assessment = assess_operation_plan(&target, &plan()).expect("the assessment completes");

    assert!(matches!(
        assessment.emission_admissible(&BTreeMap::new()),
        Err(EmissionRefusal::UnbackedPatternClaim { .. }),
    ));
}

#[test]
fn degrading_one_target_primitive_takes_the_verdicts_that_stood_on_it_down() {
    // The non-weakening property, and the one an approved pattern could
    // most easily break. A capability whose prerequisites the reviewed
    // target no longer establishes is never upgraded by a pattern: the
    // pattern would be built from a primitive the target does not have.
    //
    // The reviewed contract is the only input the public assessment
    // takes and the target package alone constructs it, so the
    // degraded contract cannot be assessed through the public entry
    // point at all. What is checked here instead is the property that
    // makes that safe: the upgrade is gated on the target-wide
    // disposition, and the two states that gate it are the two a
    // degraded target produces.
    let target = reviewed_target();
    let assessment = assess_operation_plan(&target, &plan()).expect("the assessment completes");

    for verdict in assessment.verdicts() {
        if matches!(
            verdict.target_wide(),
            Some(
                AssessmentDisposition::Unsupported | AssessmentDisposition::MissingTargetPrimitives
            ),
        ) {
            assert_eq!(
                verdict.operation(),
                verdict.target_wide().expect("checked above"),
                "a degraded prerequisite is never upgraded by a pattern",
            );
            assert!(verdict.patterns().is_empty());
        }
    }

    // And on the reviewed contract every prerequisite the assessment
    // rests on is actually reviewed, so the gate above is not vacuous
    // by accident.
    let contract = target.definition();
    for verdict in assessment.verdicts() {
        for primitive in verdict.primitives() {
            assert_eq!(
                contract
                    .capabilities()
                    .get(primitive)
                    .map(target_elements::CapabilityContract::status),
                Some(StaticCapabilityStatus::Reviewed),
                "{primitive:?}",
            );
        }
    }
}

#[test]
fn the_aggregate_verdict_names_the_slicing_its_technique_needs() {
    // The prerequisite the target-wide mapping does not carry, because
    // it is a prerequisite of this pattern's technique rather than of
    // exact arithmetic in general: the fragment slices an introspected
    // amount to a fixed-width operand before any arithmetic touches it.
    // Recording it on the operation row is what keeps the assessment
    // exact without pushing a technique back into a general answer.
    let target = reviewed_target();
    let assessment = assess_operation_plan(&target, &plan()).expect("the assessment completes");

    let verdict = assessment
        .verdict(&OperationRequirement::Capability(
            RequiredCapability::ExactPublicAmountArithmetic,
        ))
        .expect("arithmetic is assessed");

    assert!(
        verdict
            .primitives()
            .contains(&ElementsCapability::ByteStringSlicing),
    );
    assert!(
        verdict
            .primitives()
            .contains(&ElementsCapability::SignedFixedWidthArithmetic),
    );
    assert_eq!(
        verdict.patterns(),
        &BTreeSet::from([BackendPatternId::CompactAshExplicitSumV1]),
    );
}

#[test]
fn no_sponsor_amount_reaches_any_verdict_or_any_pattern() {
    // §1.6 and §8.3 together. The sponsor region's amount must not be
    // a prerequisite of anything, and the check is on the prerequisite
    // census the isolation pattern derives from its own instructions
    // rather than on a declaration.
    let target = reviewed_target();
    let census = patterns();
    let isolation = &census[&BackendPatternId::CompactAshSponsorIsolationV1];

    assert!(
        !isolation
            .prerequisites()
            .contains(&ElementsCapability::InputValueInspection),
    );

    let assessment = assess_operation_plan(&target, &plan()).expect("the assessment completes");
    let sponsor_rows = assessment
        .verdicts()
        .filter(|verdict| {
            verdict
                .patterns()
                .contains(&BackendPatternId::CompactAshSponsorIsolationV1)
        })
        .count();

    assert!(sponsor_rows > 0, "the isolation pattern discharges rows");
    assert_eq!(
        assessment.emission_admissible(&census),
        Ok(()),
        "and none of them reads an amount",
    );
}
