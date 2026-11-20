//! Focused controls and one-coordinate subjects for the dormant Clear facade.

use std::collections::BTreeSet;

use architecture::{
    DeltaCondition, ManifestError, ObjectId, OperationId, PermissionClass, RootId, RootUse,
    RootUseSpec, TagId,
};
use realization::{
    AvailabilityClass, ProofAlternativeId, ProofKind, Relation, RelationId, RelationKind,
    RelationSubject, StateField, TransactionSide,
};

use crate::{
    CompileError,
    analyzed::{ScopedAnalyzedProgram, analyze_scoped_program},
    clear_operation_plan::{ValidatedClearOperationPlan, derive_clear_plan, validate_clear_plan},
    coverage::{CoverageBoundary, CoveragePurpose, RelationMutation},
    layout::LayoutRequirement,
    placement::PlacementSearchLimits,
    requirement::ProofDisposition,
    source::{
        BurnClearCoordinate, OperandId, OperandRole, RequiredSourceKind, RequirementActivation,
        SourceRequirement,
    },
};

fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

struct Subject {
    analyzed: ScopedAnalyzedProgram,
    plan: ValidatedClearOperationPlan,
}

fn subject() -> &'static Subject {
    static SUBJECT: std::sync::OnceLock<Subject> = std::sync::OnceLock::new();
    SUBJECT.get_or_init(|| {
        let control = super::source_tests::clear_control();
        let analyzed =
            analyze_scoped_program(&control.input, limits()).expect("complete Clear analysis");
        let plan = derive_clear_plan(&analyzed).expect("explicit Clear facade");
        validate_clear_plan(&analyzed, &plan).expect("independent Clear validation");
        Subject { analyzed, plan }
    })
}

#[allow(clippy::too_many_lines)] // The three closed-law rows form one literal join.
fn assert_clear_law_requirements(plan: &ValidatedClearOperationPlan) {
    use realization::{
        ExprId, FactId,
        burn_clear_laws::{BurnClearLaw as Law, BurnClearLawKey as Key},
    };

    let source = |role, kind| (role, kind);
    let ash = |side| OperandRole::ObjectFamilyAmount {
        side,
        object: ObjectId::Ash,
    };
    let state = |field| OperandRole::ExpressionResult {
        expression: ExprId::fact(FactId::StateField {
            operation: OperationId::Clear,
            side: TransactionSide::Input,
            field,
        }),
    };
    let flow = OperandRole::BurnClear(BurnClearCoordinate::DestructionFlow);
    let output_amount = OperandRole::BurnClear(BurnClearCoordinate::DataOutputScalar(
        architecture::DataOutputKind::Destruction,
    ));
    let laws = [
        (
            RelationKind::CanonicalDeltaPolicy,
            Relation::ClearCanonicalDeltaPolicy,
            Key::CanonicalDeltaPolicy,
            Law::CanonicalDeltaPolicy,
            ProofKind::ManifestShape,
            vec![
                source(
                    OperandRole::CanonicalPartition,
                    RequiredSourceKind::AuthenticatedTransitionCertificate,
                ),
                source(flow.clone(), RequiredSourceKind::AuthenticatedOutputObject),
            ],
            vec![
                RelationMutation::MissingCanonicalDeltaFamily,
                RelationMutation::UnexpectedCanonicalDeltaFamily,
                RelationMutation::DuplicateCanonicalSourceOrDestination,
            ],
        ),
        (
            RelationKind::ClearClamp,
            Relation::ClearClamp,
            Key::ClearClamp,
            Law::ClearClamp,
            ProofKind::PublicArithmetic,
            vec![
                source(
                    ash(TransactionSide::Input),
                    RequiredSourceKind::AuthenticatedConsensusValue,
                ),
                source(state(StateField::YL), RequiredSourceKind::DerivedExpression),
                source(state(StateField::YT), RequiredSourceKind::DerivedExpression),
            ],
            vec![RelationMutation::AmountMismatch],
        ),
        (
            RelationKind::ClearFlow,
            Relation::ClearFlow,
            Key::ClearFlow,
            Law::ClearFlow,
            ProofKind::PublicArithmetic,
            vec![
                source(
                    OperandRole::CanonicalPartition,
                    RequiredSourceKind::AuthenticatedTransitionCertificate,
                ),
                source(
                    OperandRole::OpenFlowSet,
                    RequiredSourceKind::AuthenticatedTransitionCertificate,
                ),
                source(flow, RequiredSourceKind::AuthenticatedOutputObject),
                source(
                    output_amount,
                    RequiredSourceKind::AuthenticatedConsensusValue,
                ),
                source(
                    ash(TransactionSide::Input),
                    RequiredSourceKind::AuthenticatedConsensusValue,
                ),
                source(
                    ash(TransactionSide::Output),
                    RequiredSourceKind::AuthenticatedConsensusValue,
                ),
            ],
            vec![
                RelationMutation::BurnClearCoordinateChanged(BurnClearCoordinate::DestructionFlow),
                RelationMutation::AmountMismatch,
            ],
        ),
    ];
    let closed_laws = realization::burn_clear_laws::burn_clear_laws(OperationId::Clear);
    for (kind, body, key, law_body, proof_kind, sources, negatives) in laws {
        let id = RelationId::new(OperationId::Clear, kind, RelationSubject::Operation);
        let declaration = plan
            .source()
            .realization()
            .relations
            .nodes
            .iter()
            .find(|row| row.id == id)
            .expect("closed Clear declaration retained in typed source");
        assert_eq!(declaration.relation, body);
        let closed = closed_laws
            .iter()
            .find(|row| row.id.key == key)
            .expect("closed Clear law identity");
        assert_eq!(closed.body, law_body);
        assert_eq!(closed.relation_declaration().relation, body);
        let requirement = plan.relation(&id).expect("Clear law requirement");
        assert_eq!(
            requirement.proof,
            ProofDisposition::Selected {
                proof: ProofAlternativeId::new(id.clone(), proof_kind),
            }
        );
        let expected_sources = sources
            .into_iter()
            .map(|(role, kind)| SourceRequirement {
                operand: OperandId::new(id.clone(), role),
                source: kind,
                availability: AvailabilityClass::Public,
                activation: RequirementActivation::Always,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(requirement.source_requirements, expected_sources);
        let expected_coverage = std::iter::once(CoveragePurpose::ActiveAccept)
            .chain(negatives.into_iter().map(CoveragePurpose::FocusedReject))
            .collect::<BTreeSet<_>>();
        for case in plan.cases() {
            let actual = plan
                .coverage()
                .filter(|row| {
                    row.id.relation == id
                        && row.id.case == case.id
                        && row.id.boundary == CoverageBoundary::RuntimeCarrier
                })
                .filter_map(|row| match &row.id.purpose {
                    purpose @ (CoveragePurpose::ActiveAccept
                    | CoveragePurpose::FocusedReject(_)) => Some(purpose.clone()),
                    _ => None,
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(actual, expected_coverage);
        }
    }
}

#[test]
#[allow(clippy::too_many_lines)] // One accepted control feeds the selected one-coordinate subjects.
fn clear_plan_control_and_corruptions() {
    let Subject { analyzed, plan } = subject();
    assert_clear_law_requirements(plan);
    let scope = realization::RealizationScope::from_operations([OperationId::Clear])
        .expect("Clear realization scope");
    let changed_clear = |change: fn(&mut architecture::OperationSpec)| {
        let mut operations = architecture::ARCHITECTURE.operations.to_vec();
        let clear = operations
            .iter_mut()
            .find(|operation| operation.id == OperationId::Clear)
            .expect("Clear operation");
        change(clear);
        let mut architecture = architecture::ARCHITECTURE;
        architecture.operations = Box::leak(operations.into_boxed_slice());
        architecture
    };
    assert!(architecture::validate_draft(&architecture::ARCHITECTURE).is_ok());
    assert!(realization::derive(&architecture::ARCHITECTURE, scope.clone()).is_ok());

    let operator = changed_clear(|clear| clear.authorization = PermissionClass::Operator);
    assert!(architecture::validate_draft(&operator).is_ok());
    assert!(matches!(
        realization::derive(&operator, scope.clone()),
        Err(
            realization::RealizationError::ArchitectureOperationMismatch {
                operation: OperationId::Clear,
                field: realization::ArchitectureMismatchField::Authorization,
            }
        )
    ));

    let changed_tag = changed_clear(|clear| {
        let mut outputs = clear.data_outputs.to_vec();
        outputs[0].tag = TagId::Burn;
        clear.data_outputs = Box::leak(outputs.into_boxed_slice());
    });
    assert!(matches!(
        architecture::validate_draft(&changed_tag),
        Err(errors) if errors.iter().any(|error| matches!(error,
            ManifestError::InvalidOperation {
                operation: OperationId::Clear,
                reason: "destruction delta lacks exactly one matching data output",
            }
        ))
    ));

    let removed_residual_condition = changed_clear(|clear| {
        let mut deltas = clear.canonical_deltas.to_vec();
        deltas[1].condition = DeltaCondition::Always;
        clear.canonical_deltas = Box::leak(deltas.into_boxed_slice());
    });
    assert!(architecture::validate_draft(&removed_residual_condition).is_ok());
    assert!(matches!(
        realization::derive(&removed_residual_condition, scope),
        Err(
            realization::RealizationError::ArchitectureOperationMismatch {
                operation: OperationId::Clear,
                field: realization::ArchitectureMismatchField::CanonicalDelta,
            }
        )
    ));

    let inserted_resv = changed_clear(|clear| {
        let mut roots = clear.roots.to_vec();
        roots.push(RootUseSpec {
            root: RootId::Resv,
            use_kind: RootUse::Succession,
        });
        clear.roots = Box::leak(roots.into_boxed_slice());
    });
    assert!(matches!(
        architecture::validate_draft(&inserted_resv),
        Err(errors) if errors.iter().any(|error| matches!(error,
            ManifestError::InvalidOperation {
                operation: OperationId::Clear,
                reason: "succession root must have exactly one input and one output",
            }
        ))
    ));

    let burn_record = changed_clear(|clear| {
        let mut outputs = clear.data_outputs.to_vec();
        outputs[0].kind = architecture::DataOutputKind::BurnRecord;
        clear.data_outputs = Box::leak(outputs.into_boxed_slice());
    });
    assert!(matches!(
        architecture::validate_draft(&burn_record),
        Err(errors) if errors.iter().any(|error| matches!(error,
            ManifestError::InvalidOperation {
                operation: OperationId::Clear,
                reason: "burn-record data output must be tag-burn, assetless, and \
                                 bounded by BURN_RECORD_MAX",
            }
        ))
    ));
    assert_eq!(plan.operation(), OperationId::Clear);
    assert_eq!(plan.relations().count(), 44);
    assert_eq!(plan.fields().count(), 6);
    for field in StateField::ALL {
        let row = plan.field(*field).expect("one assignment per STATE field");
        assert_eq!(row.field(), *field);
        assert_eq!(row.relation().kind(), RelationKind::ClearAssignment);
        assert_eq!(row.predicate().is_none(), *field == StateField::YL);
    }
    assert!(plan.cases().count() > 0);
    assert!(plan.carriers().count() > 0);
    assert!(plan.layout().count() > 0);
    assert!(plan.coverage().count() > 0);

    let authorization = plan
        .relations()
        .find(|row| row.relation.kind() == RelationKind::Authorization)
        .expect("Clear permissionless authorization")
        .relation
        .clone();
    let mut removed_authorization = (*plan).clone();
    let row = removed_authorization
        .relations_mut()
        .get_mut(&authorization)
        .expect("authorization row");
    let source = row
        .source_requirements
        .iter()
        .find(|source| source.operand.role() == &OperandRole::ProtocolSignerSet)
        .expect("public signer absence")
        .clone();
    assert!(row.source_requirements.remove(&source));
    assert_eq!(
        validate_clear_plan(analyzed, &removed_authorization),
        Err(CompileError::TargetPlanRelationRequirementMismatch)
    );

    let mut changed_ash_availability = (*plan).clone();
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
        validate_clear_plan(analyzed, &changed_ash_availability),
        Err(CompileError::TargetPlanRelationRequirementMismatch)
    );

    let anchors = [
        crate::source::BurnClearCoordinate::ClearEventType,
        crate::source::BurnClearCoordinate::ClearDestroyedAmountEvent,
        crate::source::BurnClearCoordinate::ClearOmegaEvent,
        crate::source::BurnClearCoordinate::ClearResidualEvent,
    ]
    .into_iter()
    .map(|coordinate| {
        let role = OperandRole::BurnClear(coordinate);
        plan.coverage()
            .find(|row| match &row.id.purpose {
                CoveragePurpose::ReportAnchorRequired { operand } => operand.role() == &role,
                _ => false,
            })
            .expect("one required Clear event coordinate")
            .id
            .clone()
    })
    .collect::<Vec<_>>();
    for anchor in &anchors {
        let mut removed_anchor = (*plan).clone();
        assert!(removed_anchor.coverage_mut().remove(anchor).is_some());
        assert_eq!(
            validate_clear_plan(analyzed, &removed_anchor),
            Err(CompileError::TargetPlanCoverageCensusMismatch)
        );
    }
    let anchor = anchors.into_iter().next().expect("Clear event anchor");

    let mut changed_boundary = (*plan).clone();
    let mut row = changed_boundary
        .coverage_mut()
        .remove(&anchor)
        .expect("anchor row");
    row.id.boundary = CoverageBoundary::RuntimeCarrier;
    let mut changed_id = anchor;
    changed_id.boundary = CoverageBoundary::RuntimeCarrier;
    changed_boundary.coverage_mut().insert(changed_id, row);
    assert_eq!(
        validate_clear_plan(analyzed, &changed_boundary),
        Err(CompileError::TargetPlanCoverageCensusMismatch)
    );

    let mut removed_exit = (*plan).clone();
    removed_exit.lifecycle_mut().clear_outstanding();
    assert_eq!(
        validate_clear_plan(analyzed, &removed_exit),
        Err(CompileError::TargetPlanLifecycleMismatch)
    );

    let mut removed_carrier = (*plan).clone();
    let carrier = removed_carrier
        .carriers()
        .next()
        .expect("Clear carrier")
        .clone();
    assert!(removed_carrier.carriers_mut().remove(&carrier));
    assert_eq!(
        validate_clear_plan(analyzed, &removed_carrier),
        Err(CompileError::TargetPlanCarrierCensusMismatch)
    );

    let floor = plan
        .relations()
        .find(|row| row.relation.kind() == RelationKind::ClearFloor)
        .expect("Clear floor requirement")
        .relation
        .clone();
    let mut removed_floor = (*plan).clone();
    assert!(removed_floor.relations_mut().remove(&floor).is_some());
    assert_eq!(
        validate_clear_plan(analyzed, &removed_floor),
        Err(CompileError::TargetPlanRelationCensusMismatch { relation: floor })
    );

    let mut removed_field = (*plan).clone();
    assert!(removed_field.fields_mut().remove(&StateField::YL).is_some());
    assert_eq!(
        validate_clear_plan(analyzed, &removed_field),
        Err(CompileError::TargetPlanRelationRequirementMismatch)
    );

    let mut copied_yl = (*plan).clone();
    let yl = copied_yl
        .fields_mut()
        .get_mut(&StateField::YL)
        .expect("retained YL field row");
    yl.set_predicate(Some(realization::ExprId::relation(
        RelationId::new(
            OperationId::Clear,
            RelationKind::ClearAssignment,
            RelationSubject::StateField {
                field: StateField::YL,
            },
        ),
        realization::ExpressionRole::Predicate,
    )));
    assert_eq!(copied_yl.fields().count(), 6);
    assert_eq!(
        validate_clear_plan(analyzed, &copied_yl),
        Err(CompileError::TargetPlanRelationRequirementMismatch)
    );

    let delta = RelationId::new(
        OperationId::Clear,
        RelationKind::CanonicalDeltaPolicy,
        RelationSubject::Operation,
    );
    let mut removed_delta = (*plan).clone();
    assert!(removed_delta.relations_mut().remove(&delta).is_some());
    assert_eq!(
        validate_clear_plan(analyzed, &removed_delta),
        Err(CompileError::TargetPlanRelationCensusMismatch { relation: delta })
    );

    assert_eq!(
        plan.relations()
            .map(|row| row.relation.clone())
            .collect::<BTreeSet<_>>()
            .len(),
        44
    );
}

#[test]
fn clear_sponsor_amount_is_refused() {
    let Subject { analyzed, plan } = subject();
    assert_eq!(validate_clear_plan(analyzed, plan), Ok(()));
    let authorization = plan
        .relations()
        .find(|row| row.relation.kind() == RelationKind::Authorization)
        .expect("Clear authorization")
        .relation
        .clone();
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
    let mut sponsor_read = (*plan).clone();
    let case = plan.cases().next().expect("Clear case").id.clone();
    sponsor_read
        .layout_mut()
        .insert(LayoutRequirement::MakeSourceAvailable {
            relation: authorization,
            case,
            carrier: crate::carrier::CarrierRole::OperationGlobal {
                operation: OperationId::Clear,
                anchor: ObjectId::State,
            },
            source: sponsor,
        });
    assert_eq!(
        validate_clear_plan(analyzed, &sponsor_read),
        Err(CompileError::SponsorValueRead)
    );
}
