//! Exact announcement censuses, independent state expectations, and corruptions.

use super::{reviewed_target, target_with_status};
use crate::maturity_assessment::{assess_validated_maturity_plan, from_rows};
use crate::{
    AssessmentDisposition as D, MaturityAssessmentSet, MaturityCapabilityGroup as G,
    MaturityRequirement as R, MaturityVerdict, TapscriptError, VerdictGround as Ground,
    assess_maturity_announcement_plan,
};
use architecture::{ARCHITECTURE, OperationId};
use compiler::maturity_announcement_plan::{
    MaturityAnnouncementRepresentationPlan as Mode, ValidatedMaturityAnnouncementOperationPlan,
    plan_maturity_announcement_target_operation,
};
use compiler::operation_plan::{
    ExternalEvidenceRole as E, LayoutRequirement as L, PlacementSearchLimits,
    RequiredCapability as C,
};
use compiler::{AnalysisPolicy, CompilationScope, ProofSearchLimits, bind_input};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};
use target_elements::{
    ElementsCapability as P, StaticCapabilityStatus as S, TargetEvidenceRequirementId as Evidence,
};

static PLAN: LazyLock<ValidatedMaturityAnnouncementOperationPlan> = LazyLock::new(|| {
    let limit = |value: u64| value.try_into().unwrap();
    let operations = [
        OperationId::AnnounceMaturity,
        OperationId::CompactAsh,
        OperationId::TransferLive,
    ];
    let realization = realization::derive(
        &ARCHITECTURE,
        realization::RealizationScope::from_operations(operations).unwrap(),
    )
    .unwrap();
    let scope = CompilationScope::from_operations([OperationId::AnnounceMaturity]).unwrap();
    let policy = AnalysisPolicy::strict(ProofSearchLimits::new(limit(1_000_000), limit(10_000)));
    let input = bind_input(&ARCHITECTURE, realization, scope, policy).unwrap();
    plan_maturity_announcement_target_operation(
        &input,
        PlacementSearchLimits::new(limit(10_000_000), limit(1_000_000)),
    )
    .unwrap()
});
static ASSESSMENT: LazyLock<MaturityAssessmentSet> =
    LazyLock::new(|| assess_maturity_announcement_plan(&reviewed_target(), &PLAN).unwrap());

const CAPABILITIES: [(C, D); 8] = [
    (C::AuthenticatedObjectRecognition, D::BackendPatternRequired),
    (C::AuthenticatedFamilyCardinality, D::BackendPatternRequired),
    (
        C::AuthenticatedCanonicalPartition,
        D::BackendPatternRequired,
    ),
    (C::AuthenticatedOpenFlowPartition, D::BackendPatternRequired),
    (C::AuthenticatedRootEffects, D::BackendPatternRequired),
    (C::AuthenticatedProjectionSet, D::BackendPatternRequired),
    (C::OperatorAuthorization, D::MissingTargetPrimitives),
    (
        C::WholeTransactionValueConservation,
        D::ExternalEvidenceRequired,
    ),
];
const GROUPS: [(G, D); 18] = [
    (G::CurrentInputIndex, D::BackendPatternRequired),
    (G::InputAndOutputCount, D::BackendPatternRequired),
    (G::InputAssetAndValueInspection, D::BackendPatternRequired),
    (
        G::InputAndOutputProgramInspection,
        D::BackendPatternRequired,
    ),
    (G::TransactionVersionAndLockTime, D::BackendPatternRequired),
    (G::ByteEquality, D::BackendPatternRequired),
    (
        G::CheckedArithmeticAndComparisons,
        D::BackendPatternRequired,
    ),
    (G::ByteSlicingAndConcatenation, D::BackendPatternRequired),
    (G::StreamingSha256, D::BackendPatternRequired),
    (G::TapleafAndTapbranchHashing, D::MissingTargetPrimitives),
    (G::TweakVerification, D::BackendPatternRequired),
    (G::XOnlyKeyEncoding, D::MissingTargetPrimitives),
    (G::SignatureVerification, D::BackendPatternRequired),
    (G::SelectedSighashSemantics, D::MissingTargetPrimitives),
    (G::ScriptPathExecution, D::BackendStructural),
    (
        G::TargetTransactionConservation,
        D::ExternalEvidenceRequired,
    ),
    (G::FeeRoleRecognition, D::MissingTargetPrimitives),
    (G::ResourceLimits, D::BackendStructural),
];

fn rows() -> Vec<(Mode, MaturityVerdict)> {
    Mode::ALL
        .iter()
        .flat_map(|mode| ASSESSMENT.verdicts(*mode).cloned().map(|row| (*mode, row)))
        .collect()
}

#[test]
fn full_verdict_sets_equal_the_published_requirements_and_literal_groups() {
    for projection in PLAN.representations() {
        let expected = projection
            .capabilities()
            .map(R::Capability)
            .chain(projection.layout().cloned().map(R::Layout))
            .chain(projection.external_evidence().map(R::ExternalEvidence))
            .chain(GROUPS.map(|(group, _)| R::Group(group)))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            ASSESSMENT
                .verdicts(projection.plan())
                .map(|row| row.requirement().clone())
                .collect::<BTreeSet<_>>(),
            expected
        );
        assert_eq!(
            (
                projection.capabilities().count(),
                projection.layout().count(),
                projection.external_evidence().count()
            ),
            (8, 59, 2)
        );
        assert_eq!(ASSESSMENT.projection(projection.plan()).unwrap().len(), 87);
    }
    assert_eq!(ASSESSMENT.len(), 174);
    assert!(!ASSESSMENT.is_empty());
}

#[test]
fn every_published_capability_has_its_literal_disposition() {
    for mode in Mode::ALL {
        let actual = ASSESSMENT
            .verdicts(*mode)
            .filter_map(|row| match row.requirement() {
                R::Capability(capability) => Some((*capability, row.operation())),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, CAPABILITIES);
    }
}

#[test]
fn every_layout_row_has_its_literal_category_disposition() {
    for mode in Mode::ALL {
        let mut counts = [0; 7];
        for row in PLAN.projection(*mode).unwrap().layout() {
            let (category, expected) = match row {
                L::CanonicalCoordinator { .. } => (0, D::BackendStructural),
                L::AuthenticateFamilyCensus { .. } => (1, D::BackendPatternRequired),
                L::CompleteAndDisjointFamilies { .. } => (2, D::BackendPatternRequired),
                L::IsolateSponsorRegion { .. } => (3, D::BackendPatternRequired),
                L::EnforceRepresentation { .. } => (4, D::BackendPatternRequired),
                L::MakeSourceAvailable { .. } => (5, D::BackendPatternRequired),
                L::SecretFreeOperationPath { .. } => (6, D::BackendPatternRequired),
            };
            counts[category] += 1;
            assert_eq!(
                ASSESSMENT
                    .verdict(*mode, &R::Layout(row.clone()))
                    .unwrap()
                    .operation(),
                expected,
                "{row:?}"
            );
        }
        assert_eq!(counts, [3, 8, 8, 4, 1, 35, 0]);
    }
}

#[test]
fn every_group_has_its_literal_disposition_and_census_order() {
    assert_eq!(G::ALL, &GROUPS.map(|(group, _)| group));
    for mode in Mode::ALL {
        for (group, expected) in GROUPS {
            assert_eq!(
                ASSESSMENT
                    .verdict(*mode, &R::Group(group))
                    .unwrap()
                    .operation(),
                expected
            );
        }
    }
}

#[test]
fn both_state_censuses_are_literal() {
    for mode in Mode::ALL {
        assert_eq!(
            ASSESSMENT.state_census(*mode),
            BTreeMap::from([
                (D::MissingTargetPrimitives, 5),
                (D::BackendPatternRequired, 73),
                (D::BackendStructural, 5),
                (D::ExternalEvidenceRequired, 4),
            ])
        );
    }
}

#[test]
fn both_ground_censuses_are_literal() {
    for mode in Mode::ALL {
        assert_eq!(
            ASSESSMENT.ground_census(*mode),
            BTreeMap::from([
                (Ground::TargetWideAssessment, 5),
                (Ground::NoApprovedPattern, 73),
                (Ground::CompilerOrAbiObligation, 5),
                (Ground::ExternalTargetClaim, 4),
            ])
        );
    }
}

#[test]
fn operator_evidence_keeps_its_external_signature_and_sighash_discharge_path() {
    assert_eq!(
        *PLAN.operator().evidence(),
        realization::ExternalEvidenceRequirement::OperatorAuthorization {
            operation: OperationId::AnnounceMaturity
        }
    );
    for mode in Mode::ALL {
        assert_eq!(
            PLAN.projection(*mode)
                .unwrap()
                .external_evidence()
                .collect::<Vec<_>>(),
            [E::SubstrateConservation, E::OperatorAuthorization]
        );
        let row = ASSESSMENT
            .verdict(*mode, &R::ExternalEvidence(E::OperatorAuthorization))
            .unwrap();
        assert_eq!(row.operation(), D::ExternalEvidenceRequired);
        assert_eq!(row.ground(), Ground::ExternalTargetClaim);
        assert_eq!(
            row.evidence(),
            &BTreeSet::from([Evidence::SignatureSemantics, Evidence::SighashSemantics])
        );
    }
}

#[test]
fn no_pilot_pattern_is_a_maturity_completion_claim() {
    // No STATE pattern exists; compact and live patterns cannot complete these rows.
    for (_, row) in rows() {
        assert_ne!(row.operation(), D::CompleteBackendPattern);
        assert_ne!(row.ground(), Ground::ApprovedPattern);
        assert_ne!(row.ground(), Ground::StructuralAbsence);
        if row.operation() == D::BackendPatternRequired {
            assert_eq!(row.ground(), Ground::NoApprovedPattern);
        }
    }
}

#[test]
fn the_two_incomplete_prerequisites_block_both_dependent_rows() {
    let target = reviewed_target();
    for primitive in [P::OutputCommittingSighash, P::InputCommitmentControl] {
        assert_eq!(
            target.validated().definition().capabilities()[&primitive].status(),
            S::Incomplete
        );
        for mode in Mode::ALL {
            for requirement in [
                R::Capability(C::OperatorAuthorization),
                R::Group(G::SelectedSighashSemantics),
            ] {
                let row = ASSESSMENT.verdict(*mode, &requirement).unwrap();
                assert!(row.primitives().contains(&primitive));
                assert_eq!(row.operation(), D::MissingTargetPrimitives);
            }
        }
    }
}

#[test]
fn groups_without_registry_entries_are_missing_not_inferred_from_encodings() {
    for group in [
        G::TapleafAndTapbranchHashing,
        G::XOnlyKeyEncoding,
        G::FeeRoleRecognition,
    ] {
        assert_eq!(group.target_capabilities(), []);
        for mode in Mode::ALL {
            assert_eq!(
                ASSESSMENT
                    .verdict(*mode, &R::Group(group))
                    .unwrap()
                    .operation(),
                D::MissingTargetPrimitives
            );
        }
    }
}

#[test]
fn duplicate_rows_of_every_family_are_refused_in_each_representation() {
    for mode in Mode::ALL {
        for requirement in [
            R::Capability(C::OperatorAuthorization),
            R::Layout(
                PLAN.projection(*mode)
                    .unwrap()
                    .layout()
                    .next()
                    .unwrap()
                    .clone(),
            ),
            R::ExternalEvidence(E::OperatorAuthorization),
            R::Group(G::ByteEquality),
        ] {
            let mut corrupted = rows();
            corrupted.push((
                *mode,
                ASSESSMENT.verdict(*mode, &requirement).unwrap().clone(),
            ));
            assert_eq!(
                from_rows(&PLAN, corrupted),
                Err(TapscriptError::DuplicateOperationRequirement)
            );
        }
    }
}

#[test]
fn every_missing_row_is_refused_with_its_exact_identity() {
    for (mode, removed) in rows() {
        let corrupted = rows().into_iter().filter(|(candidate, row)| {
            *candidate != mode || row.requirement() != removed.requirement()
        });
        assert_eq!(
            from_rows(&PLAN, corrupted),
            Err(TapscriptError::MaturityAssessmentCensusMismatch {
                representation: mode,
                missing: vec![removed.requirement().clone()],
                unexpected: vec![],
            })
        );
    }
}

#[test]
fn an_extra_unpublished_capability_is_refused() {
    for mode in Mode::ALL {
        let mut corrupted = rows();
        let mut extra = ASSESSMENT
            .verdict(*mode, &R::Capability(C::OperatorAuthorization))
            .unwrap()
            .clone();
        extra.restate(
            R::Capability(C::RefundAuthorization),
            D::MissingTargetPrimitives,
        );
        corrupted.push((*mode, extra));
        assert_eq!(
            from_rows(&PLAN, corrupted),
            Err(TapscriptError::MaturityAssessmentCensusMismatch {
                representation: *mode,
                missing: vec![],
                unexpected: vec![R::Capability(C::RefundAuthorization)],
            })
        );
    }
}

#[test]
fn representation_groups_agree_including_all_obligations() {
    ASSESSMENT.check_group_agreement().unwrap();
    for group in G::ALL {
        assert_eq!(
            ASSESSMENT.verdict(Mode::Explicit, &R::Group(*group)),
            ASSESSMENT.verdict(Mode::PublicCommitted, &R::Group(*group))
        );
    }
}

#[test]
fn corrupted_group_agreement_has_a_typed_error() {
    let mut corrupted = rows();
    let requirement = R::Group(G::ByteEquality);
    let (_, row) = corrupted
        .iter_mut()
        .find(|(mode, row)| *mode == Mode::PublicCommitted && row.requirement() == &requirement)
        .unwrap();
    row.restate(requirement, D::CompleteBackendPattern);
    assert_eq!(
        from_rows(&PLAN, corrupted),
        Err(TapscriptError::MaturityGroupDisagreement {
            group: G::ByteEquality
        })
    );
}

#[test]
fn target_degradation_is_pointwise_non_promoting_for_every_registry_capability() {
    for primitive in P::ALL {
        for status in [S::Incomplete, S::Unsupported] {
            let target = target_with_status(*primitive, status);
            let degraded = assess_validated_maturity_plan(&target, &PLAN).unwrap();
            for mode in Mode::ALL {
                assert_eq!(degraded.verdicts(*mode).count(), 87);
                for row in degraded.verdicts(*mode) {
                    let baseline = ASSESSMENT.verdict(*mode, row.requirement()).unwrap();
                    assert!(
                        row.operation() <= baseline.operation(),
                        "{primitive:?}: {row:?}"
                    );
                }
                assert_eq!(
                    degraded
                        .verdict(*mode, &R::ExternalEvidence(E::OperatorAuthorization))
                        .unwrap()
                        .operation(),
                    D::ExternalEvidenceRequired
                );
            }
        }
    }
}

#[test]
fn a_specific_unsupported_primitive_actually_lowers_affected_verdicts() {
    let target = target_with_status(P::OutputProgramInspection, S::Unsupported);
    let degraded = assess_validated_maturity_plan(&target, &PLAN).unwrap();
    for mode in Mode::ALL {
        for requirement in [
            R::Capability(C::AuthenticatedObjectRecognition),
            R::Group(G::InputAndOutputProgramInspection),
        ] {
            assert_eq!(
                ASSESSMENT.verdict(*mode, &requirement).unwrap().operation(),
                D::BackendPatternRequired
            );
            assert_eq!(
                degraded.verdict(*mode, &requirement).unwrap().operation(),
                D::Unsupported
            );
        }
    }
}

#[test]
fn the_private_core_and_public_entry_point_assess_the_same_reviewed_definition() {
    assert_eq!(
        assess_validated_maturity_plan(reviewed_target().validated(), &PLAN).unwrap(),
        *ASSESSMENT
    );
    let row = ASSESSMENT
        .verdict(
            Mode::Explicit,
            &R::Capability(C::AuthenticatedFamilyCardinality),
        )
        .unwrap();
    assert_eq!(row.target_wide(), Some(D::BackendStructural));
    assert!(!row.structural().is_empty());
}
