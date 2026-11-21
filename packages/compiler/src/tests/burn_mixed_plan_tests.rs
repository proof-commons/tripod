//! Focused controls and corruptions for the dormant mixed Burn facade.

use std::collections::BTreeSet;

use architecture::{AssetId, ObjectId, OperationId};
use realization::{
    AvailabilityClass, ExternalEvidenceRequirement, ProofKind, RelationId, RelationKind,
    RelationSubject, RepresentationMode, TransactionSide,
};

use crate::{
    CompileError,
    analyzed::{ScopedAnalyzedProgram, analyze_scoped_program},
    burn_operation_plan::{
        ExternalEvidenceRole, ValidatedBurnOperationPlan, derive_mixed_burn_plan,
        validate_mixed_burn_output_selection, validate_mixed_burn_plan,
    },
    lifecycle::{LifecycleExitStatus, RepresentationChoiceId},
    placement::PlacementSearchLimits,
    requirement::ProofDisposition,
    source::{OperandRole, RequiredSourceKind, RequirementActivation},
};

const fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

struct Subject {
    analyzed: ScopedAnalyzedProgram,
    plan: ValidatedBurnOperationPlan,
}

fn subject() -> Subject {
    let control = super::source_tests::burn_mixed_control();
    let analyzed =
        analyze_scoped_program(&control.input, limits()).expect("complete Burn analysis");
    let plan = derive_mixed_burn_plan(&analyzed).expect("mixed Burn facade");
    validate_mixed_burn_plan(&analyzed, &plan).expect("fresh mixed validation");
    Subject { analyzed, plan }
}

const fn choice(object: ObjectId, side: Option<TransactionSide>) -> RepresentationChoiceId {
    RepresentationChoiceId {
        operation: OperationId::Burn,
        object,
        side,
    }
}

const fn conservation() -> RelationId {
    RelationId::new(
        OperationId::Burn,
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    )
}

fn amount_source(
    plan: &ValidatedBurnOperationPlan,
    side: TransactionSide,
    object: ObjectId,
) -> crate::source::SourceRequirement {
    plan.relation(&conservation())
        .expect("Burn U conservation")
        .source_requirements
        .iter()
        .find(|row| {
            matches!(
                row.operand.role(),
                OperandRole::ObjectFamilyAmount {
                    side: row_side,
                    object: row_object,
                } if *row_side == side && *row_object == object
            )
        })
        .expect("one side-qualified conservation source")
        .clone()
}

#[test]
#[allow(clippy::too_many_lines)] // One complete control feeds exact tuple and join corruptions.
fn mixed_burn_plan_preserves_public_ash_and_private_change() {
    let Subject { analyzed, plan } = subject();
    assert_eq!(plan.operation(), OperationId::Burn);
    assert_eq!(plan.relations().count(), 32);
    assert_eq!(plan.representation().selected().count(), 3);
    for (object, side, mode) in [
        (
            ObjectId::ReceiptLive,
            Some(TransactionSide::Input),
            RepresentationMode::Explicit,
        ),
        (ObjectId::Ash, None, RepresentationMode::Explicit),
        (
            ObjectId::ReceiptLive,
            Some(TransactionSide::Output),
            RepresentationMode::PrivateCommitted,
        ),
    ] {
        assert_eq!(
            plan.representation().selection(&choice(object, side)),
            Some(mode)
        );
    }
    assert_eq!(
        validate_mixed_burn_output_selection(
            &analyzed,
            &plan,
            RepresentationMode::PrivateCommitted
        ),
        Ok(()),
    );
    assert_eq!(
        validate_mixed_burn_output_selection(&analyzed, &plan, RepresentationMode::Explicit),
        Err(CompileError::TargetPlanRepresentationMismatch {
            object: ObjectId::ReceiptLive,
        }),
    );

    let relation = plan.relation(&conservation()).expect("Burn U conservation");
    assert!(matches!(
        &relation.proof,
        ProofDisposition::Selected { proof }
            if proof.proof() == ProofKind::ConfidentialConservation
    ));
    for (side, object, expected) in [
        (
            TransactionSide::Input,
            ObjectId::ReceiptLive,
            RequiredSourceKind::AuthenticatedConsensusValue,
        ),
        (
            TransactionSide::Output,
            ObjectId::Ash,
            RequiredSourceKind::AuthenticatedConsensusValue,
        ),
        (
            TransactionSide::Output,
            ObjectId::ReceiptLive,
            RequiredSourceKind::AuthenticatedCommitmentRelation,
        ),
    ] {
        let row = amount_source(&plan, side, object);
        assert_eq!(row.source, expected);
        assert_eq!(row.availability, AvailabilityClass::Public);
        assert_eq!(row.activation, RequirementActivation::Always);
    }
    assert!(plan.lifecycle().outstanding().any(|row| {
        row.object == ObjectId::ReceiptLive
            && row.representation == RepresentationMode::PrivateCommitted
            && row.exit == OperationId::Burn
            && row.status == LifecycleExitStatus::UnavailableInCompilerScope
    }));
    assert!(!plan.lifecycle().release_complete());

    let mut changed_ash = plan.clone();
    let ash = amount_source(&plan, TransactionSide::Output, ObjectId::Ash);
    let row = changed_ash
        .relations_mut()
        .get_mut(&conservation())
        .expect("conservation relation");
    assert!(row.source_requirements.remove(&ash));
    let mut changed = ash;
    changed.availability = AvailabilityClass::Operator;
    row.source_requirements.insert(changed);
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed_ash),
        Err(CompileError::TargetPlanRelationRequirementMismatch),
    );

    // These two rows are selected from the produced plan. They are pins
    // for omission/refusal, not independently derived layout or coverage.
    let mut changed_layout = plan.clone();
    let layout = changed_layout.layout().next().expect("layout row").clone();
    assert!(changed_layout.layout_mut().remove(&layout));
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed_layout),
        Err(CompileError::TargetPlanLayoutCensusMismatch),
    );
    let mut changed_coverage = plan.clone();
    let coverage = changed_coverage
        .coverage()
        .next()
        .expect("coverage row")
        .id
        .clone();
    assert!(changed_coverage.coverage_mut().remove(&coverage).is_some());
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed_coverage),
        Err(CompileError::TargetPlanCoverageCensusMismatch),
    );
    let mut changed_lifecycle = plan;
    changed_lifecycle.lifecycle_mut().clear_outstanding();
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed_lifecycle),
        Err(CompileError::TargetPlanLifecycleMismatch),
    );
}

#[test]
fn mixed_burn_plan_rejects_lost_commitment_evidence() {
    let Subject { analyzed, plan } = subject();
    let requirement = ExternalEvidenceRequirement::ConfidentialValueConservation {
        operation: OperationId::Burn,
        asset: AssetId::U,
    };
    assert_eq!(
        plan.relation(&conservation())
            .expect("Burn U conservation")
            .external_evidence,
        BTreeSet::from([requirement.clone()]),
    );
    assert!(
        plan.external_evidence()
            .any(|role| role == ExternalEvidenceRole::ConfidentialValueConservation)
    );

    let mut changed_source = plan.clone();
    let commitment = amount_source(&plan, TransactionSide::Output, ObjectId::ReceiptLive);
    assert!(
        changed_source
            .relations_mut()
            .get_mut(&conservation())
            .expect("conservation relation")
            .source_requirements
            .remove(&commitment)
    );
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed_source),
        Err(CompileError::TargetPlanRelationRequirementMismatch),
    );

    let mut changed_requirement = plan.clone();
    assert!(
        changed_requirement
            .relations_mut()
            .get_mut(&conservation())
            .expect("conservation relation")
            .external_evidence
            .remove(&requirement)
    );
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed_requirement),
        Err(CompileError::TargetPlanRelationRequirementMismatch),
    );

    let mut changed_role = plan;
    assert!(
        changed_role
            .external_evidence_mut()
            .remove(&ExternalEvidenceRole::ConfidentialValueConservation)
    );
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed_role),
        Err(CompileError::TargetPlanEvidenceCensusMismatch),
    );
}

#[test]
fn mixed_burn_plan_rejects_input_output_aliasing() {
    let Subject { analyzed, plan } = subject();
    let input = choice(ObjectId::ReceiptLive, Some(TransactionSide::Input));
    let output = choice(ObjectId::ReceiptLive, Some(TransactionSide::Output));
    assert_eq!(
        plan.representation().selection(&input),
        Some(RepresentationMode::Explicit)
    );
    assert_eq!(
        plan.representation().selection(&output),
        Some(RepresentationMode::PrivateCommitted),
    );

    let mut changed = plan.clone();
    let mode = changed
        .selections_mut()
        .remove(&output)
        .expect("output change selection");
    changed
        .selections_mut()
        .insert(choice(ObjectId::ReceiptLive, None), mode);
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed),
        Err(CompileError::TargetPlanRepresentationMismatch {
            object: ObjectId::ReceiptLive,
        }),
    );

    let mut changed_source = plan;
    let output_amount = amount_source(
        &changed_source,
        TransactionSide::Output,
        ObjectId::ReceiptLive,
    );
    let row = changed_source
        .relations_mut()
        .get_mut(&conservation())
        .expect("conservation relation");
    assert!(row.source_requirements.remove(&output_amount));
    let mut aliased = output_amount;
    aliased.operand = crate::source::OperandId::new(
        conservation(),
        OperandRole::ObjectFamilyAmount {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    );
    row.source_requirements.insert(aliased);
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed_source),
        Err(CompileError::TargetPlanRelationRequirementMismatch),
    );
}
