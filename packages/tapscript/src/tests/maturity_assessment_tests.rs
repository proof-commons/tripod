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

use super::state_program_tests::fixtures;
use crate::maturity_assessment::{
    assess_maturity_program_parts, capability_components, group_components, layout_components,
    maturity_relation_carrier, project_maturity_carrier_parts,
};
use crate::{
    MaturityCarrier as Carrier, MaturityCarrierRefusalReason as Refusal, StateAnnouncementId as A,
    StateOperatorPatternId, StatePatternId as B, StateProgramComponent as Component,
    assess_maturity_announcement_program, maturity_announcement_record_census,
    project_maturity_carriers,
};
use architecture::ObjectId;
use compiler::operation_plan::RequiredSourceKind as Source;
use realization::{
    RelationId, RelationKind as K, RelationSubject as Subject, TransactionSide as Side,
};

static RECORD_ASSESSMENT: LazyLock<MaturityAssessmentSet> = LazyLock::new(|| {
    assess_maturity_announcement_program(&reviewed_target(), &PLAN, &fixtures().program).unwrap()
});

const RECORD_GROUPS: [(G, D, Ground); 18] = [
    (
        G::CurrentInputIndex,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::InputAndOutputCount,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::InputAssetAndValueInspection,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::InputAndOutputProgramInspection,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::TransactionVersionAndLockTime,
        D::BackendPatternRequired,
        Ground::NoApprovedPattern,
    ),
    (
        G::ByteEquality,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::CheckedArithmeticAndComparisons,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::ByteSlicingAndConcatenation,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::StreamingSha256,
        D::BackendPatternRequired,
        Ground::NoApprovedPattern,
    ),
    (
        G::TapleafAndTapbranchHashing,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::TweakVerification,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::XOnlyKeyEncoding,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::SignatureVerification,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::SelectedSighashSemantics,
        D::MissingTargetPrimitives,
        Ground::TargetWideAssessment,
    ),
    (
        G::ScriptPathExecution,
        D::BackendStructural,
        Ground::CompilerOrAbiObligation,
    ),
    (
        G::TargetTransactionConservation,
        D::ExternalEvidenceRequired,
        Ground::ExternalTargetClaim,
    ),
    (
        G::FeeRoleRecognition,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        G::ResourceLimits,
        D::BackendStructural,
        Ground::CompilerOrAbiObligation,
    ),
];
const RECORD_CAPABILITIES: [(C, D, Ground); 8] = [
    (
        C::AuthenticatedObjectRecognition,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        C::AuthenticatedFamilyCardinality,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        C::AuthenticatedCanonicalPartition,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        C::AuthenticatedOpenFlowPartition,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        C::AuthenticatedRootEffects,
        D::BackendPatternRequired,
        Ground::NoApprovedPattern,
    ),
    (
        C::AuthenticatedProjectionSet,
        D::CompleteBackendPattern,
        Ground::ApprovedPattern,
    ),
    (
        C::OperatorAuthorization,
        D::MissingTargetPrimitives,
        Ground::TargetWideAssessment,
    ),
    (
        C::WholeTransactionValueConservation,
        D::ExternalEvidenceRequired,
        Ground::ExternalTargetClaim,
    ),
];

fn components() -> BTreeSet<Component> {
    fixtures().program.components().keys().copied().collect()
}

fn assess_parts(available: &BTreeSet<Component>) -> MaturityAssessmentSet {
    let record = &fixtures().program;
    assess_maturity_program_parts(
        reviewed_target().validated(),
        &PLAN,
        available,
        record.prerequisites(),
        record.metadata(),
    )
    .unwrap()
}

fn expected_layout(layout: &L) -> (D, Ground) {
    match layout {
        L::CanonicalCoordinator { .. } => (D::BackendStructural, Ground::CompilerOrAbiObligation),
        L::AuthenticateFamilyCensus { .. }
        | L::CompleteAndDisjointFamilies { .. }
        | L::IsolateSponsorRegion { .. }
        | L::EnforceRepresentation { .. } => (D::CompleteBackendPattern, Ground::ApprovedPattern),
        L::MakeSourceAvailable { source, .. } => match source.source {
            Source::AuthenticatedInputObject
            | Source::AuthenticatedOutputObject
            | Source::AuthenticatedFamilyCensus
            | Source::RuntimeArchitectureBound
            | Source::OperatorWitness
            | Source::PublicConstructionData => {
                (D::CompleteBackendPattern, Ground::ApprovedPattern)
            }
            Source::ExternalEvidence => (D::ExternalEvidenceRequired, Ground::ExternalTargetClaim),
            _ => (D::BackendPatternRequired, Ground::NoApprovedPattern),
        },
        L::SecretFreeOperationPath { .. } => (D::BackendPatternRequired, Ground::NoApprovedPattern),
    }
}

#[test]
fn record_groups_match_every_literal_disposition_and_ground() {
    for mode in Mode::ALL {
        for (group, disposition, ground) in RECORD_GROUPS {
            let row = RECORD_ASSESSMENT.verdict(*mode, &R::Group(group)).unwrap();
            assert_eq!(
                (row.operation(), row.ground()),
                (disposition, ground),
                "{group:?}"
            );
        }
    }
}

#[test]
fn record_capabilities_match_every_literal_disposition_and_ground() {
    for mode in Mode::ALL {
        for (capability, disposition, ground) in RECORD_CAPABILITIES {
            let row = RECORD_ASSESSMENT
                .verdict(*mode, &R::Capability(capability))
                .unwrap();
            assert_eq!(
                (row.operation(), row.ground()),
                (disposition, ground),
                "{capability:?}"
            );
        }
    }
}

#[test]
fn record_layouts_match_the_literal_kind_and_source_table() {
    for mode in Mode::ALL {
        for layout in PLAN.projection(*mode).unwrap().layout() {
            let row = RECORD_ASSESSMENT
                .verdict(*mode, &R::Layout(layout.clone()))
                .unwrap();
            assert_eq!(
                (row.operation(), row.ground()),
                expected_layout(layout),
                "{layout:?}"
            );
        }
    }
}

#[test]
fn record_groups_and_carriers_agree_across_representations() {
    RECORD_ASSESSMENT.check_group_agreement().unwrap();
    let projection = project_maturity_carriers(&PLAN, &fixtures().program).unwrap();
    assert_eq!(
        projection.projection(Mode::Explicit),
        projection.projection(Mode::PublicCommitted)
    );
    assert_eq!(
        RECORD_ASSESSMENT.state_census(Mode::Explicit),
        RECORD_ASSESSMENT.state_census(Mode::PublicCommitted)
    );
}

#[test]
fn record_does_not_complete_unreviewed_sighash_or_operator_capability() {
    for mode in Mode::ALL {
        for requirement in [
            R::Group(G::SelectedSighashSemantics),
            R::Capability(C::OperatorAuthorization),
        ] {
            assert_eq!(
                RECORD_ASSESSMENT.verdict(*mode, &requirement),
                ASSESSMENT.verdict(*mode, &requirement)
            );
        }
    }
}

#[test]
fn degraded_registry_still_lowers_record_backed_verdicts() {
    let record = &fixtures().program;
    for status in [S::Incomplete, S::Unsupported] {
        let target = target_with_status(P::OutputProgramInspection, status);
        let degraded = assess_maturity_program_parts(
            &target,
            &PLAN,
            &components(),
            record.prerequisites(),
            record.metadata(),
        )
        .unwrap();
        for mode in Mode::ALL {
            for requirement in [
                R::Group(G::InputAndOutputProgramInspection),
                R::Capability(C::AuthenticatedObjectRecognition),
            ] {
                assert_eq!(
                    RECORD_ASSESSMENT
                        .verdict(*mode, &requirement)
                        .unwrap()
                        .operation(),
                    D::CompleteBackendPattern
                );
                assert_eq!(
                    degraded.verdict(*mode, &requirement).unwrap().operation(),
                    if status == S::Unsupported {
                        D::Unsupported
                    } else {
                        D::MissingTargetPrimitives
                    }
                );
            }
        }
    }
}

#[test]
fn every_removed_component_leaves_its_promoted_rows_at_the_standing_result() {
    for removed in components() {
        let mut available = components();
        available.remove(&removed);
        let assessment = assess_parts(&available);
        for mode in Mode::ALL {
            for row in RECORD_ASSESSMENT.verdicts(*mode) {
                let carrying = match row.requirement() {
                    R::Group(group) => group_components(*group),
                    R::Capability(capability) => capability_components(*capability),
                    R::Layout(layout) => layout_components(layout),
                    R::ExternalEvidence(_) => &[],
                };
                let expected = if carrying.contains(&removed) {
                    ASSESSMENT.verdict(*mode, row.requirement()).unwrap()
                } else {
                    row
                };
                assert_eq!(
                    assessment.verdict(*mode, row.requirement()),
                    Some(expected),
                    "{removed:?}"
                );
            }
        }
    }
}

#[test]
fn removing_a_registry_prerequisite_blocks_every_dependent_promotion() {
    let record = &fixtures().program;
    for removed in record.prerequisites() {
        let mut prerequisites = record.prerequisites().clone();
        prerequisites.remove(removed);
        let assessment = assess_maturity_program_parts(
            reviewed_target().validated(),
            &PLAN,
            &components(),
            &prerequisites,
            record.metadata(),
        )
        .unwrap();
        for mode in Mode::ALL {
            for row in RECORD_ASSESSMENT
                .verdicts(*mode)
                .filter(|row| row.primitives().contains(removed))
            {
                assert_eq!(
                    assessment.verdict(*mode, row.requirement()),
                    ASSESSMENT.verdict(*mode, row.requirement())
                );
            }
        }
    }
}

#[test]
fn source_rows_require_their_exact_metadata_source_kind() {
    let record = &fixtures().program;
    for removed in &record.metadata().sources {
        let mut metadata = record.metadata().clone();
        metadata.sources.remove(removed);
        let assessment = assess_maturity_program_parts(
            reviewed_target().validated(),
            &PLAN,
            &components(),
            record.prerequisites(),
            &metadata,
        )
        .unwrap();
        for mode in Mode::ALL {
            for layout in PLAN.projection(*mode).unwrap().layout() {
                if let L::MakeSourceAvailable { source, .. } = layout
                    && source.source == *removed
                {
                    let requirement = R::Layout(layout.clone());
                    assert_eq!(
                        assessment.verdict(*mode, &requirement),
                        ASSESSMENT.verdict(*mode, &requirement)
                    );
                }
            }
        }
    }
}

#[test]
fn external_rows_remain_byte_for_byte_equal_and_promotions_retain_obligations() {
    for mode in Mode::ALL {
        for row in RECORD_ASSESSMENT.verdicts(*mode) {
            let standing = ASSESSMENT.verdict(*mode, row.requirement()).unwrap();
            assert_eq!(row.primitives(), standing.primitives());
            assert_eq!(row.structural(), standing.structural());
            assert_eq!(row.target_wide(), standing.target_wide());
            if row.operation() == D::CompleteBackendPattern {
                let mut evidence = standing.evidence().clone();
                evidence.extend(&fixtures().program.metadata().evidence);
                assert_eq!(row.evidence(), &evidence);
            } else {
                assert_eq!(row, standing);
            }
        }
    }
}

fn expected_completed(mode: Mode) -> BTreeSet<R> {
    RECORD_GROUPS
        .into_iter()
        .filter(|(_, disposition, _)| *disposition == D::CompleteBackendPattern)
        .map(|(group, _, _)| R::Group(group))
        .chain(
            RECORD_CAPABILITIES
                .into_iter()
                .filter(|(_, disposition, _)| *disposition == D::CompleteBackendPattern)
                .map(|(capability, _, _)| R::Capability(capability)),
        )
        .chain(
            PLAN.projection(mode)
                .unwrap()
                .layout()
                .filter(|layout| expected_layout(layout).0 == D::CompleteBackendPattern)
                .cloned()
                .map(R::Layout),
        )
        .collect()
}

#[test]
fn record_census_sets_are_exact_disjoint_and_cover_every_requirement() {
    let census =
        maturity_announcement_record_census(&reviewed_target(), &PLAN, &fixtures().program)
            .unwrap();
    assert_eq!(census.len(), 2);
    for mode in Mode::ALL {
        let expected = expected_completed(*mode);
        let all = ASSESSMENT
            .verdicts(*mode)
            .map(|row| row.requirement().clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(census[mode].completed, expected);
        assert_eq!(
            census[mode].remaining,
            all.difference(&expected).cloned().collect()
        );
        assert!(census[mode].completed.is_disjoint(&census[mode].remaining));
        assert_eq!(
            census[mode].completed.len() + census[mode].remaining.len(),
            87
        );
    }
}

fn expected_emitted_relations() -> BTreeMap<RelationId, Component> {
    let mut rows = BTreeMap::new();
    let mut insert = |kind, subject, component| {
        rows.insert(
            RelationId::new(OperationId::AnnounceMaturity, kind, subject),
            component,
        );
    };
    for side in [Side::Input, Side::Output] {
        insert(
            K::Cardinality,
            Subject::ObjectFamily {
                side,
                object: ObjectId::State,
            },
            Component::Structural(B::StateCardinalityV1),
        );
        insert(
            K::Cardinality,
            Subject::ObjectFamily {
                side,
                object: ObjectId::PlainLbtc,
            },
            Component::Structural(B::StateSponsorIsolationV1),
        );
        insert(
            K::Recognition,
            Subject::ObjectFamily {
                side,
                object: ObjectId::PlainLbtc,
            },
            Component::Structural(B::StateSponsorIsolationV1),
        );
        insert(
            K::AllowedObjectFamilies,
            Subject::TransactionSide { side },
            Component::Structural(B::StateIssuanceAbsenceV1),
        );
    }
    for (side, component) in [
        (
            Side::Input,
            Component::Structural(B::StateInputRecognitionV1),
        ),
        (
            Side::Output,
            Component::Semantic(A::SuccessorReconstruction),
        ),
    ] {
        insert(
            K::Recognition,
            Subject::ObjectFamily {
                side,
                object: ObjectId::State,
            },
            component,
        );
    }
    rows.extend(expected_policy_relations());
    rows
}

fn expected_policy_relations() -> BTreeMap<RelationId, Component> {
    let mut rows = BTreeMap::new();
    let mut insert = |kind, subject, component| {
        rows.insert(
            RelationId::new(OperationId::AnnounceMaturity, kind, subject),
            component,
        );
    };
    for kind in [K::SponsorIsolation, K::SponsorEnvelopeMultiplicity] {
        insert(
            kind,
            Subject::Sponsor,
            Component::Structural(B::StateSponsorIsolationV1),
        );
    }
    for (kind, component) in [
        (
            K::Authorization,
            Component::Operator(StateOperatorPatternId::OperatorAuthorizationV1),
        ),
        (
            K::OpenFlowPolicy,
            Component::Structural(B::StateSponsorIsolationV1),
        ),
        (
            K::CanonicalDeltaPolicy,
            Component::Structural(B::StateIssuanceAbsenceV1),
        ),
        (
            K::RootPolicy,
            Component::Semantic(A::MetadataAuthentication),
        ),
        (K::ProjectionPolicy, Component::Semantic(A::CopyThrough)),
    ] {
        insert(kind, Subject::Operation, component);
    }
    insert(
        K::Representation,
        Subject::Representation {
            object: ObjectId::State,
        },
        Component::Semantic(A::SuccessorReconstruction),
    );
    for exit in [
        OperationId::AdmitDeposits,
        OperationId::Cycle,
        OperationId::Redeem,
        OperationId::ReceiptRelabel,
        OperationId::Clear,
        OperationId::AnnounceMaturity,
    ] {
        insert(
            K::Lifecycle,
            Subject::LifecycleExit {
                object: ObjectId::State,
                exit,
            },
            Component::Semantic(if exit == OperationId::AnnounceMaturity {
                A::LeadWindow
            } else {
                A::MaturityPredecessor
            }),
        );
    }
    rows
}

#[test]
fn carriers_cover_the_literal_twenty_six_relations_and_each_emitted_component_exists() {
    let projection = project_maturity_carriers(&PLAN, &fixtures().program).unwrap();
    let expected = expected_emitted_relations();
    assert_eq!(expected.len(), 24);
    for mode in Mode::ALL {
        let rows = projection.projection(*mode).unwrap();
        assert_eq!(rows.len(), 26);
        assert_eq!(
            rows.keys().collect::<BTreeSet<_>>(),
            PLAN.projection(*mode)
                .unwrap()
                .relations()
                .map(|row| &row.relation)
                .collect()
        );
        let emitted = rows
            .iter()
            .filter_map(|(relation, carrier)| match carrier {
                Carrier::Emitted(component) => {
                    assert!(fixtures().program.components().contains_key(component));
                    Some((relation.clone(), *component))
                }
                Carrier::External(_) => None,
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(emitted, expected);
    }
}

#[test]
fn external_carriers_are_exact_realization_requirements_except_verified_authorization() {
    let projection = project_maturity_carriers(&PLAN, &fixtures().program).unwrap();
    for mode in Mode::ALL {
        for row in PLAN.projection(*mode).unwrap().relations() {
            let carrier = &projection.projection(*mode).unwrap()[&row.relation];
            if &row.relation == PLAN.operator().authorization() {
                assert_eq!(
                    carrier,
                    &Carrier::Emitted(Component::Operator(
                        StateOperatorPatternId::OperatorAuthorizationV1
                    ))
                );
                assert_eq!(
                    row.external_evidence,
                    BTreeSet::from([PLAN.operator().evidence().clone()])
                );
            } else if let Some(external) = row.external_evidence.first() {
                assert_eq!(carrier, &Carrier::External(external.clone()));
                assert_eq!(row.external_evidence.len(), 1);
            } else {
                assert!(matches!(carrier, Carrier::Emitted(_)));
            }
        }
    }
}

#[test]
fn removing_each_component_refuses_exactly_its_literal_relations_by_name() {
    let expected = expected_emitted_relations();
    for removed in components() {
        let mut available = components();
        available.remove(&removed);
        for mode in Mode::ALL {
            let mut refused = BTreeSet::new();
            for row in PLAN.projection(*mode).unwrap().relations() {
                if let Err(error) = maturity_relation_carrier(&PLAN, row, &available) {
                    assert_eq!(error.relation, row.relation);
                    assert_eq!(error.reason, Refusal::MissingComponent(removed));
                    refused.insert(error.relation);
                }
            }
            let expected = expected
                .iter()
                .filter(|(_, component)| **component == removed)
                .map(|(relation, _)| relation.clone())
                .collect::<BTreeSet<_>>();
            assert_eq!(refused, expected);
            let projected = project_maturity_carrier_parts(&PLAN, &available);
            if let Some(first) = refused.first() {
                assert_eq!(projected.unwrap_err().relation, *first);
            } else {
                projected.unwrap();
            }
        }
    }
}

#[test]
fn unplaceable_relations_and_ambiguous_external_sets_are_named_refusals() {
    let mut row = PLAN
        .projection(Mode::Explicit)
        .unwrap()
        .relations()
        .next()
        .unwrap()
        .clone();
    row.relation = RelationId::new(
        OperationId::AnnounceMaturity,
        K::ExpressionPredicate,
        Subject::Operation,
    );
    row.external_evidence.clear();
    let error = maturity_relation_carrier(&PLAN, &row, &components()).unwrap_err();
    assert_eq!(error.relation, row.relation);
    assert_eq!(error.reason, Refusal::Unmapped);
    row.external_evidence.extend([
        realization::ExternalEvidenceRequirement::OperatorAuthorization {
            operation: OperationId::AnnounceMaturity,
        },
        realization::ExternalEvidenceRequirement::SubstrateConservation {
            operation: OperationId::AnnounceMaturity,
            asset: architecture::AssetId::Lbtc,
        },
    ]);
    let error = maturity_relation_carrier(&PLAN, &row, &components()).unwrap_err();
    assert_eq!(error.relation, row.relation);
    assert_eq!(error.reason, Refusal::MultipleExternalRequirements);
}

#[test]
fn paired_carrier_omissions_name_only_relations_of_the_removed_components() {
    let expected = expected_emitted_relations();
    let all = components();
    for first in &all {
        for second in all.iter().filter(|second| *second > first) {
            let mut available = all.clone();
            available.remove(first);
            available.remove(second);
            for mode in Mode::ALL {
                for row in PLAN.projection(*mode).unwrap().relations() {
                    let result = maturity_relation_carrier(&PLAN, row, &available);
                    if let Some(component) = expected
                        .get(&row.relation)
                        .filter(|component| *component == first || *component == second)
                    {
                        let error = result.unwrap_err();
                        assert_eq!(error.relation, row.relation);
                        assert_eq!(error.reason, Refusal::MissingComponent(*component));
                    } else {
                        assert_eq!(result, maturity_relation_carrier(&PLAN, row, &all));
                    }
                }
            }
        }
    }
}

#[test]
fn carrier_refusal_declaration_has_exact_exercised_and_unreachable_census() {
    super::state_program_tests::assert_refusal_census(
        include_str!("../maturity_assessment.rs"),
        "MaturityCarrierRefusalReason",
        &[
            (
                "Unmapped",
                unplaceable_relations_and_ambiguous_external_sets_are_named_refusals,
            ),
            (
                "MissingComponent",
                removing_each_component_refuses_exactly_its_literal_relations_by_name,
            ),
            (
                "MultipleExternalRequirements",
                unplaceable_relations_and_ambiguous_external_sets_are_named_refusals,
            ),
        ],
        &[(
            "RepresentationDisagreement",
            "Compiler corruption accessors are private and unavailable in dependent-crate tests; needs a projection-over-rows seam.",
        )],
    );
}

#[test]
fn assessment_error_declarations_match_the_exercised_corruptions() {
    let declared = super::state_program_tests::declared_variants(
        include_str!("../error.rs"),
        "TapscriptError",
    );
    let relevant: BTreeSet<_> = declared
        .into_iter()
        .filter(|name| name.starts_with("Maturity") || *name == "DuplicateOperationRequirement")
        .collect();
    let cases: [(&str, fn()); 3] = [
        (
            "DuplicateOperationRequirement",
            duplicate_rows_of_every_family_are_refused_in_each_representation,
        ),
        (
            "MaturityAssessmentCensusMismatch",
            every_missing_row_is_refused_with_its_exact_identity,
        ),
        (
            "MaturityGroupDisagreement",
            corrupted_group_agreement_has_a_typed_error,
        ),
    ];
    let reached = cases
        .into_iter()
        .map(|(name, test)| {
            test();
            name
        })
        .collect();
    assert_eq!(relevant, reached);
}
