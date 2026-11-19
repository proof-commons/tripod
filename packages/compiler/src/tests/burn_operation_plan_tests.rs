//! Focused controls and one-coordinate subjects for the dormant Burn facade.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use realization::{AvailabilityClass, RelationKind, RepresentationMode};

use crate::{
    CompileError,
    analyzed::{ScopedAnalyzedProgram, analyze_scoped_program},
    burn_operation_plan::{
        ValidatedBurnOperationPlan, derive_burn_plan, validate_burn_output_selection,
        validate_burn_plan,
    },
    coverage::{CoverageBoundary, CoveragePurpose},
    engine_requirement::{Bundle, SelectionError, Table},
    layout::LayoutRequirement,
    placement::PlacementSearchLimits,
    source::{OperandId, OperandRole, RequiredSourceKind, SourceRequirement},
};

fn limits() -> PlacementSearchLimits {
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
    let control = super::source_tests::burn_control();
    let analyzed =
        analyze_scoped_program(&control.input, limits()).expect("complete Burn analysis");
    let plan = derive_burn_plan(&analyzed).expect("explicit Burn facade");
    validate_burn_plan(&analyzed, &plan).expect("independent Burn validation");
    Subject { analyzed, plan }
}

#[test]
#[allow(clippy::too_many_lines)] // One accepted control feeds the selected one-coordinate subjects.
fn burn_plan_control_and_corruptions() {
    let Subject { analyzed, plan } = subject();
    assert_eq!(plan.operation(), OperationId::Burn);
    assert_eq!(plan.relations().count(), 32);
    assert!(plan.cases().count() > 0);
    assert!(plan.carriers().count() > 0);
    assert!(plan.layout().count() > 0);
    assert!(plan.coverage().count() > 0);

    let authorization = plan
        .relations()
        .find(|row| row.relation.kind() == RelationKind::Authorization)
        .expect("Burn owner authorization")
        .relation
        .clone();
    let mut removed_authorization = plan.clone();
    let row = removed_authorization
        .relations_mut()
        .get_mut(&authorization)
        .expect("authorization row");
    let source = row
        .source_requirements
        .iter()
        .find(|source| source.source == RequiredSourceKind::InputOwnerWitness)
        .expect("owner witness")
        .clone();
    assert!(row.source_requirements.remove(&source));
    assert_eq!(
        validate_burn_plan(&analyzed, &removed_authorization),
        Err(CompileError::TargetPlanRelationRequirementMismatch)
    );

    let mut changed_ash_availability = plan.clone();
    let source_row = changed_ash_availability
        .relations_mut()
        .values_mut()
        .flat_map(|row| row.source_requirements.iter())
        .find(|source| {
            source.availability == AvailabilityClass::Public
                && matches!(
                    source.operand.role(),
                    OperandRole::ObjectFamilyAmount {
                        object: ObjectId::Ash,
                        ..
                    }
                )
        })
        .expect("public ASH amount")
        .clone();
    let owner = source_row.operand.relation().clone();
    let row = changed_ash_availability
        .relations_mut()
        .get_mut(&owner)
        .expect("source owner");
    assert!(row.source_requirements.remove(&source_row));
    let mut changed = source_row;
    changed.availability = AvailabilityClass::Operator;
    row.source_requirements.insert(changed);
    assert_eq!(
        validate_burn_plan(&analyzed, &changed_ash_availability),
        Err(CompileError::TargetPlanRelationRequirementMismatch)
    );

    let anchors = [
        crate::source::BurnClearCoordinate::BurnEventType,
        crate::source::BurnClearCoordinate::BurnFreshAmountEvent,
    ]
    .into_iter()
    .map(|coordinate| {
        let role = OperandRole::BurnClear(coordinate);
        plan.coverage()
            .find(|row| match &row.id.purpose {
                CoveragePurpose::ReportAnchorRequired { operand } => operand.role() == &role,
                _ => false,
            })
            .expect("one required Burn event coordinate")
            .id
            .clone()
    })
    .collect::<Vec<_>>();
    for anchor in &anchors {
        let mut removed_anchor = plan.clone();
        assert!(removed_anchor.coverage_mut().remove(anchor).is_some());
        assert_eq!(
            validate_burn_plan(&analyzed, &removed_anchor),
            Err(CompileError::TargetPlanCoverageCensusMismatch)
        );
    }
    let anchor = anchors.into_iter().next().expect("Burn event anchor");

    let mut changed_boundary = plan.clone();
    let mut row = changed_boundary
        .coverage_mut()
        .remove(&anchor)
        .expect("anchor row");
    row.id.boundary = CoverageBoundary::RuntimeCarrier;
    let mut changed_id = anchor;
    changed_id.boundary = CoverageBoundary::RuntimeCarrier;
    changed_boundary.coverage_mut().insert(changed_id, row);
    assert_eq!(
        validate_burn_plan(&analyzed, &changed_boundary),
        Err(CompileError::TargetPlanCoverageCensusMismatch)
    );

    let mut removed_exit = plan.clone();
    removed_exit.lifecycle_mut().clear_outstanding();
    assert_eq!(
        validate_burn_plan(&analyzed, &removed_exit),
        Err(CompileError::TargetPlanLifecycleMismatch)
    );

    let mut removed_carrier = plan.clone();
    let carrier = removed_carrier
        .carriers()
        .next()
        .expect("Burn carrier")
        .clone();
    assert!(removed_carrier.carriers_mut().remove(&carrier));
    assert_eq!(
        validate_burn_plan(&analyzed, &removed_carrier),
        Err(CompileError::TargetPlanCarrierCensusMismatch)
    );

    let sponsor = SourceRequirement {
        operand: OperandId::new(
            authorization.clone(),
            OperandRole::ObjectFamilyAmount {
                side: realization::TransactionSide::Input,
                object: ObjectId::PlainLbtc,
            },
        ),
        source: RequiredSourceKind::AuthenticatedConsensusValue,
        availability: AvailabilityClass::Public,
        activation: crate::source::RequirementActivation::Always,
    };
    let mut sponsor_read = plan.clone();
    let case = plan.cases().next().expect("Burn case").id.clone();
    sponsor_read
        .layout_mut()
        .insert(LayoutRequirement::MakeSourceAvailable {
            relation: authorization,
            case,
            carrier: crate::carrier::CarrierRole::OperationGlobal {
                operation: OperationId::Burn,
                anchor: ObjectId::ReceiptLive,
            },
            source: sponsor,
        });
    assert_eq!(
        validate_burn_plan(&analyzed, &sponsor_read),
        Err(CompileError::SponsorValueRead)
    );

    let relation = plan
        .relations()
        .next()
        .expect("Burn relation")
        .relation
        .clone();
    let table = Table {
        relation: crate::engine_attestation::relation_key(&relation),
        required: false,
        choices: BTreeMap::from([(None, Ok(Bundle::default()))]),
    };
    let selected = BTreeMap::new();
    assert_eq!(
        crate::engine_requirement::derive(std::slice::from_ref(&table), &selected)
            .expect("single relation control")
            .len(),
        1
    );
    assert_eq!(
        crate::engine_requirement::derive(&[table.clone(), table.clone()], &selected),
        Err(SelectionError::Duplicate(table.relation))
    );
    assert_eq!(
        plan.relations()
            .map(|row| row.relation.clone())
            .collect::<BTreeSet<_>>()
            .len(),
        32
    );
}

#[test]
fn burn_selection_refuses_private_change() {
    let Subject { analyzed, plan } = subject();
    assert_eq!(
        validate_burn_output_selection(&analyzed, &plan, RepresentationMode::Explicit),
        Ok(())
    );
    assert_eq!(
        validate_burn_output_selection(&analyzed, &plan, RepresentationMode::PrivateCommitted),
        Err(CompileError::TargetPlanRepresentationMismatch {
            object: ObjectId::ReceiptLive,
        })
    );
}
