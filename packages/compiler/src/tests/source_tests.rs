//! Operand, capability, and source-requirement tests (Guide-3 §17.1–2).

use std::collections::BTreeSet;

use architecture::{ObjectId, OperationId};
use realization::{
    AvailabilityClass, ProofKind, Relation, RelationDeclaration, RelationId, RelationKind,
    RelationSubject,
};

use architecture::{
    ObjectId::{PlainLbtc, State},
    OperationId::AnnounceMaturity,
};

use super::{announcement_input, bound_input};
use crate::{
    CompileError,
    capability::{CapabilityView, RequiredCapability},
    relation::build_relation_analysis,
    source::{
        OperandRole, RequiredSourceKind, RequirementActivation, derive_source_requirements,
        is_sponsor_amount_operand, relation_operands,
    },
};

pub(super) struct BurnControl {
    pub input: crate::BoundCompilerInput,
    pub relations: crate::relation::CompilerRelationAnalysis,
    pub candidate: crate::proof::ProofPlanCandidate,
}

pub(super) fn burn_control() -> BurnControl {
    burn_control_with_transfer(realization::RepresentationMode::Explicit)
}

pub(super) fn burn_control_with_transfer(
    transfer_mode: realization::RepresentationMode,
) -> BurnControl {
    let operations = [OperationId::TransferLive, OperationId::Burn];
    let realization = realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::from_operations(operations).expect("scope"),
    )
    .expect("complete realization");
    let input = crate::input::bind_attestation_input(
        &architecture::ARCHITECTURE,
        realization,
        crate::CompilationScope::from_operations(operations).expect("compiler scope"),
        super::test_policy(),
    )
    .expect("validated internal input");
    let projection = input.realization().project();
    let relations = crate::engine_attestation::lower_relations(
        input.scope().operations(),
        &projection.relations.nodes,
        &projection.relations.edges,
        &projection.lifecycle,
    )
    .analyze()
    .expect("complete declaration records");
    crate::sponsor_region::validate_decidable_sponsor_regions(
        &relations,
        input.scope().operations(),
    )
    .expect("decidable sponsor role");
    let constructibility =
        crate::constructibility::build_constructibility_analysis(&input).expect("constructibility");
    let lifecycle =
        crate::lifecycle::build_lifecycle_analysis(&input, &relations).expect("lifecycle");
    let lowered = crate::engine_attestation::proof::lower_prepared(
        &input,
        &relations,
        &constructibility,
        &lifecycle,
    )
    .expect("proof tables");
    let candidate = lowered
        .enumerate(
            &CapabilityView::Unconstrained,
            input.policy().proof_search_limits,
            &mut crate::proof::ProofSearchReport::default(),
        )
        .expect("feasible internal Burn proof")
        .into_iter()
        .find(|candidate| {
            candidate.representations.iter().any(|(choice, mode)| {
                choice.operation == OperationId::Burn
                    && choice.side == Some(realization::TransactionSide::Output)
                    && *mode == realization::RepresentationMode::Explicit
            }) && candidate.representations.iter().any(|(choice, mode)| {
                choice.operation == OperationId::TransferLive
                    && choice.side.is_none()
                    && choice.object == ObjectId::ReceiptLive
                    && *mode == transfer_mode
            })
        })
        .expect("selected Burn output choice");
    BurnControl {
        input,
        relations,
        candidate,
    }
}

#[test]
#[allow(clippy::too_many_lines)] // The three literal Burn body tables form one focused contract.
fn burn_bodies_have_exact_owned_tables() {
    use crate::coverage::{
        CoverageBoundary as B, CoveragePurpose, EvidenceRole, ProjectionSubject,
    };
    use crate::source::BurnClearCoordinate as C;

    let control = burn_control();
    let cases = crate::case::execution_cases(&control.relations, &control.candidate)
        .expect("complete Burn cases");
    let plans = crate::placement::classify_relation_cases(&control.relations, &cases)
        .expect("complete body classification");
    let burn = control
        .relations
        .graph
        .node_weights()
        .filter(|node| node.source.id.operation() == OperationId::Burn)
        .map(|node| &node.source)
        .collect::<Vec<_>>();
    let binding = burn
        .iter()
        .find(|row| matches!(row.relation, Relation::BurnBinding))
        .expect("Burn binding declaration");
    let data = burn
        .iter()
        .find(|row| matches!(row.relation, Relation::DataOutputs { .. }))
        .expect("Burn data output declaration");
    let sides = burn
        .iter()
        .filter(|row| matches!(row.relation, Relation::SideRepresentation { .. }))
        .collect::<Vec<_>>();
    assert_eq!(sides.len(), 2);
    assert_eq!(
        crate::requirement::relation_objects(OperationId::Burn, &data.relation),
        BTreeSet::new()
    );
    assert_eq!(binding.proof_alternatives.len(), 1);
    assert_eq!(data.proof_alternatives.len(), 1);
    assert!(sides.iter().all(|row| row.proof_alternatives.is_empty()));

    let sources = derive_source_requirements(binding, ProofKind::ManifestShape)
        .expect("Burn binding sources");
    let anchors = crate::coverage::report_anchors(&binding.id);
    assert_eq!(anchors.len(), 2);
    assert_eq!(
        anchors
            .iter()
            .map(|operand| operand.role().clone())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            OperandRole::BurnClear(C::BurnEventType),
            OperandRole::BurnClear(C::BurnFreshAmountEvent),
        ])
    );
    assert!(
        sources
            .iter()
            .filter(|row| anchors.contains(&row.operand))
            .all(|row| {
                row.source == RequiredSourceKind::ExternalEvidence
                    && row.availability == AvailabilityClass::Public
                    && row.activation == RequirementActivation::Always
            })
    );
    let data_sources =
        derive_source_requirements(data, ProofKind::ManifestShape).expect("Burn data sources");
    let data_coordinates = data_sources
        .iter()
        .filter_map(|row| match row.operand.role() {
            OperandRole::BurnClear(coordinate) => Some((*coordinate, row.source)),
            _ => None,
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let record = architecture::DataOutputKind::BurnRecord;
    assert_eq!(
        data_coordinates,
        std::collections::BTreeMap::from([
            (
                C::DataOutputSchema(record),
                RequiredSourceKind::AuthenticatedOutputObject
            ),
            (
                C::DataOutputFamilyCount(record),
                RequiredSourceKind::AuthenticatedOutputObject
            ),
            (
                C::DataOutputTag(record),
                RequiredSourceKind::AuthenticatedOutputObject
            ),
            (
                C::DataOutputAsset(record),
                RequiredSourceKind::AuthenticatedOutputObject
            ),
            (
                C::DataOutputCondition(record),
                RequiredSourceKind::AuthenticatedOutputObject
            ),
            (
                C::DataOutputScalar(record),
                RequiredSourceKind::AuthenticatedConsensusValue
            ),
            (
                C::RecordOrdinal,
                RequiredSourceKind::AuthenticatedOutputObject
            ),
            (
                C::RecordAddress,
                RequiredSourceKind::AuthenticatedOutputObject
            ),
            (
                C::RecordFreshAsh,
                RequiredSourceKind::AuthenticatedOutputObject
            ),
            (
                C::TransactionContext,
                RequiredSourceKind::AuthenticatedTransitionCertificate
            ),
            (
                C::CertificateContext,
                RequiredSourceKind::AuthenticatedTransitionCertificate
            ),
        ])
    );
    assert!(data_sources.iter().any(|row| row.operand.role()
        == &OperandRole::RuntimeBound {
            bound: architecture::BoundId::BurnRecordMax
        }));

    for case in cases
        .iter()
        .filter(|case| case.id.operation == OperationId::Burn)
    {
        for row in &sides {
            let plan = plans
                .iter()
                .find(|plan| plan.relation == row.id && plan.case == case.id)
                .expect("side plan");
            assert_eq!(
                plan.boundaries,
                BTreeSet::from([B::CompilerStatic, B::BackendStructural])
            );
        }
        let data_plan = plans
            .iter()
            .find(|plan| plan.relation == data.id && plan.case == case.id)
            .expect("data plan");
        assert_eq!(data_plan.boundaries, BTreeSet::from([B::BackendStructural]));
        let data_coverage =
            crate::engine_attestation::coverage::derive_relation_coverage(data, data_plan)
                .expect("data coverage");
        assert!(
            data_coverage
                .positive
                .iter()
                .any(|row| row.id.boundary == B::BackendStructural
                    && data_coordinates.keys().all(|coordinate| row
                        .operands
                        .iter()
                        .any(|operand| operand.role() == &OperandRole::BurnClear(*coordinate))))
        );
        let binding_plan = plans
            .iter()
            .find(|plan| plan.relation == binding.id && plan.case == case.id)
            .expect("binding plan");
        assert_eq!(
            binding_plan.boundaries,
            BTreeSet::from([B::RuntimeCarrier, B::ReportObligation])
        );
        assert!(
            binding_plan
                .runtime_requirements
                .iter()
                .flat_map(|row| &row.sources)
                .all(|row| !anchors.contains(&row.operand))
        );
        let covered =
            crate::engine_attestation::coverage::derive_relation_coverage(binding, binding_plan)
                .expect("binding coverage");
        assert!(
            covered
                .positive
                .iter()
                .filter(|row| row.id.boundary == B::RuntimeCarrier)
                .all(|row| row
                    .operands
                    .iter()
                    .all(|operand| !anchors.contains(operand)))
        );
        assert_eq!(
            covered
                .positive
                .iter()
                .filter(|row| matches!(row.role, EvidenceRole::ReportAnchor { .. }))
                .count(),
            2
        );
        assert_eq!(
            covered
                .negative
                .iter()
                .filter(|row| matches!(row.role, EvidenceRole::ReportAnchor { .. }))
                .count(),
            2
        );
        assert!(
            covered
                .positive
                .iter()
                .filter(|row| row.id.boundary == B::ReportObligation)
                .all(|row| matches!(row.id.purpose, CoveragePurpose::ReportAnchorRequired { .. }))
        );
        let projection = crate::coverage::projection_requirement(
            &covered.key(),
            &covered,
            binding_plan,
            B::ReportObligation,
        )
        .expect("complete report projection");
        assert_eq!(
            projection.subject,
            ProjectionSubject::ReportAnchors {
                operands: anchors.clone()
            }
        );
        assert_eq!(
            projection.operands,
            anchors.iter().cloned().collect::<Vec<_>>()
        );
        assert_eq!(
            projection.sources,
            [] as [crate::source::SourceRequirement; 0]
        );
        assert!(!projection.compare_relation_verdict);
    }

    let operands = relation_operands(binding).expect("Burn operands");
    let table = crate::engine_source::SourceTable {
        relation: crate::engine_attestation::relation_key(&binding.id),
        proof_namespace: crate::engine_identity::ATTESTATION,
        proof_kind: ProofKind::ManifestShape as u16,
        expected: operands
            .iter()
            .map(crate::engine_attestation::source::operand_key)
            .collect(),
        rows: sources
            .iter()
            .map(crate::engine_attestation::source::row_key)
            .collect(),
    };
    let mut changed = crate::engine_source::SourceRecords {
        tables: vec![table],
        regions: Vec::new(),
    };
    assert!(crate::engine_source::analyze_sources(&changed).is_ok());
    let foreign = crate::source::OperandId::new(
        binding.id.clone(),
        OperandRole::BurnClear(C::ClearEventType),
    );
    changed.tables[0].rows[0].operand = crate::engine_attestation::source::operand_key(&foreign);
    assert_eq!(
        crate::engine_source::analyze_sources(&changed),
        Err(crate::engine_source::SourceError::ForeignOperand(
            crate::engine_attestation::source::operand_key(&foreign)
        ))
    );
}

#[test]
fn side_admission_and_sponsor_opacity() {
    let control = burn_control();
    let lifecycle = crate::lifecycle::build_lifecycle_analysis(&control.input, &control.relations)
        .expect("validated side choices");
    let input = lifecycle
        .choices
        .iter()
        .find(|choice| {
            choice.id.operation == OperationId::Burn
                && choice.id.side == Some(realization::TransactionSide::Input)
                && choice.id.object == ObjectId::ReceiptLive
        })
        .expect("Burn input choice");
    assert_eq!(
        input.candidates,
        vec![realization::RepresentationMode::Explicit]
    );
    let output = lifecycle
        .choices
        .iter()
        .find(|choice| {
            choice.id.operation == OperationId::Burn
                && choice.id.side == Some(realization::TransactionSide::Output)
                && choice.id.object == ObjectId::ReceiptLive
        })
        .expect("Burn output choice");
    assert!(
        output
            .candidates
            .contains(&realization::RepresentationMode::PrivateCommitted)
    );

    let construction = crate::constructibility::build_constructibility_analysis(&control.input)
        .expect("constructibility");
    let requirements = crate::engine_attestation::requirement::derive(
        &control.relations,
        &construction,
        &control.candidate,
    )
    .expect("complete requirement control");
    let binding = requirements
        .values()
        .find(|row| {
            row.relation.operation() == OperationId::Burn
                && row.relation.kind() == RelationKind::BurnBinding
        })
        .expect("binding requirements");
    assert_eq!(
        crate::requirement::validate_sponsor_erasure(binding),
        Ok(())
    );
    let mut changed = (*binding).clone();
    let mut row = (*changed
        .source_requirements
        .iter()
        .next()
        .expect("source row"))
    .clone();
    row.operand = crate::source::OperandId::new(
        binding.relation.clone(),
        OperandRole::ObjectFamilyAmount {
            side: realization::TransactionSide::Input,
            object: ObjectId::PlainLbtc,
        },
    );
    changed.source_requirements.insert(row);
    assert_eq!(
        crate::requirement::validate_sponsor_erasure(&changed),
        Err(CompileError::SponsorValueRead)
    );
}

#[test]
fn active_carriers_and_layout_join() {
    let control = burn_control();
    let limits = crate::placement::PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(100_000).expect("nonzero"),
    );
    let placement = crate::analyzed_operation::analyze_operation_placements(
        &control.relations,
        &control.candidate,
        OperationId::Burn,
        limits,
    )
    .expect("Burn reaches carrier and layout analysis");
    let binding = placement
        .relation_case_plans
        .iter()
        .find(|plan| {
            plan.relation.operation() == OperationId::Burn
                && plan.relation.kind() == RelationKind::BurnBinding
        })
        .expect("binding plan");
    let anchors = crate::coverage::report_anchors(&binding.relation);
    let eligible = placement
        .eligibility
        .iter()
        .find(|row| row.relation == binding.relation)
        .expect("active binding carrier");
    assert!(
        eligible
            .eligible
            .iter()
            .flat_map(|entry| &entry.routings)
            .all(|routing| !anchors.contains(&routing.source.operand))
    );
    assert!(placement.layout_requirements.iter().all(|row| {
        !matches!(row, crate::layout::LayoutRequirement::MakeSourceAvailable { source, .. }
            if anchors.contains(&source.operand))
    }));
    let original = placement.placements.first().expect("feasible placement");
    assert_eq!(
        crate::placement::validate_placement(
            &placement.relation_case_plans,
            &placement.eligibility,
            original
        ),
        Ok(())
    );
    let mut changed = (*original).clone();
    let assigned = changed
        .assignments
        .iter_mut()
        .find(|row| row.relation == binding.relation)
        .expect("binding assignment");
    assigned.carriers[0].carrier = crate::carrier::CarrierRole::BackendStructural {
        operation: OperationId::Burn,
    };
    assert!(matches!(
        crate::placement::validate_placement(
            &placement.relation_case_plans,
            &placement.eligibility,
            &changed
        ),
        Err(CompileError::UnpermittedCarrierPlacement { .. })
    ));
}

fn declaration(
    operation: OperationId,
    kind: RelationKind,
    relation: Relation,
) -> RelationDeclaration {
    RelationDeclaration {
        id: RelationId::new(operation, kind, RelationSubject::Operation),
        relation,
        proof_alternatives: BTreeSet::new(),
    }
}

#[test]
fn operand_codes_are_injective_and_preserve_pilots() {
    use crate::source::BurnClearCoordinate as C;
    use architecture::DataOutputKind;

    let relation = declaration(
        OperationId::CompactAsh,
        RelationKind::Authorization,
        Relation::PermissionlessAuthorization,
    );
    let row = derive_source_requirements(&relation, ProofKind::PublicConstructibility)
        .expect("pilot source")
        .pop()
        .expect("pilot operand");
    let pilot = crate::engine_attestation::source::operand_key(&row.operand);
    assert_eq!(pilot.role, 4);
    assert_eq!(pilot.payload, crate::engine_source::Payload::None);

    let coordinates = [
        C::DataOutputSchema(DataOutputKind::BurnRecord),
        C::DataOutputSchema(DataOutputKind::Destruction),
        C::DataOutputFamilyCount(DataOutputKind::BurnRecord),
        C::DataOutputFamilyCount(DataOutputKind::Destruction),
        C::DataOutputTag(DataOutputKind::BurnRecord),
        C::DataOutputTag(DataOutputKind::Destruction),
        C::DataOutputAsset(DataOutputKind::BurnRecord),
        C::DataOutputAsset(DataOutputKind::Destruction),
        C::DataOutputCondition(DataOutputKind::BurnRecord),
        C::DataOutputCondition(DataOutputKind::Destruction),
        C::RecordOrdinal,
        C::RecordAddress,
        C::DataOutputScalar(DataOutputKind::BurnRecord),
        C::DataOutputScalar(DataOutputKind::Destruction),
        C::RecordFreshAsh,
        C::DestructionFlow,
        C::TransactionContext,
        C::CertificateContext,
        C::BurnEventType,
        C::ClearEventType,
        C::BurnFreshAmountEvent,
        C::ClearDestroyedAmountEvent,
        C::ClearOmegaEvent,
        C::ClearResidualEvent,
    ];
    let keys = coordinates
        .iter()
        .map(|coordinate| {
            crate::engine_attestation::source::operand_key(&crate::source::OperandId::new(
                relation.id.clone(),
                OperandRole::BurnClear(*coordinate),
            ))
        })
        .collect::<Vec<_>>();
    assert_eq!(keys.len(), keys.iter().collect::<BTreeSet<_>>().len());
    assert!(keys.iter().all(|key| key.role == 17));
    assert!(keys.iter().all(|key| match &key.payload {
        crate::engine_source::Payload::Fact(fact) => fact.subject[0] > 11,
        _ => false,
    }));

    let table = crate::engine_source::SourceTable {
        relation: pilot.relation.clone(),
        proof_namespace: crate::engine_identity::ATTESTATION,
        proof_kind: ProofKind::PublicConstructibility as u16,
        expected: vec![pilot],
        rows: vec![crate::engine_attestation::source::row_key(&row)],
    };
    let mut records = crate::engine_source::SourceRecords {
        tables: vec![table],
        regions: Vec::new(),
    };
    assert!(crate::engine_source::analyze_sources(&records).is_ok());
    records.tables[0].rows[0].operand = keys[0].clone();
    assert_eq!(
        crate::engine_source::analyze_sources(&records),
        Err(crate::engine_source::SourceError::ForeignOperand(
            keys[0].clone()
        )),
    );
}

#[test]
fn source_lowering_uses_body_rules_after_the_binding_seam() {
    let control = declaration(
        OperationId::AnnounceMaturity,
        RelationKind::Authorization,
        Relation::PermissionlessAuthorization,
    );
    assert_eq!(
        relation_operands(&control).expect("supported owner"),
        vec![crate::source::OperandId::new(
            control.id.clone(),
            OperandRole::ProtocolSignerSet,
        )],
    );
    assert!(derive_source_requirements(&control, ProofKind::PublicConstructibility).is_ok());
    for operation in [OperationId::Burn, OperationId::Clear] {
        let mut subject = control.clone();
        subject.id = RelationId::new(operation, control.id.kind(), control.id.subject().clone());
        assert_eq!(
            relation_operands(&subject),
            relation_operands(&control).map(|roles| roles
                .into_iter()
                .map(|role| {
                    crate::source::OperandId::new(subject.id.clone(), role.role().clone())
                })
                .collect()),
        );
        assert!(derive_source_requirements(&subject, ProofKind::PublicConstructibility).is_ok());
    }
}

// --- capability view (§17.1) ---

#[test]
fn state_field_keys_retain_the_literal_field_codes() {
    use realization::{FactId, StateField, TransactionSide};
    let expected = [0, 1, 2, 3, 4, 5];
    let actual = StateField::ALL
        .iter()
        .map(|field| crate::engine_attestation::state_field_code(*field))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    for (field, code) in StateField::ALL.iter().zip(expected) {
        for (side, side_code) in [(TransactionSide::Input, 0), (TransactionSide::Output, 1)] {
            let fact = FactId::StateField {
                operation: OperationId::Clear,
                side,
                field: *field,
            };
            assert_eq!(
                crate::engine_attestation::constructibility::fact_key(&fact),
                vec![2, 12, side_code, code],
            );
        }
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Keep each Clear body's control beside every in-crate refusal.
fn clear_bodies_refuse_each_synthetic_compiler_entry() {
    use realization::burn_clear_laws::burn_clear_laws;

    let input = bound_input(&[OperationId::CompactAsh]);
    let expressions = crate::expression::build_expression_analysis(&input).expect("expressions");
    let construction =
        crate::constructibility::build_constructibility_analysis(&input).expect("construction");
    let candidates = crate::proof::enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("proof candidates");
    let mut candidate = candidates.candidates.into_iter().next().expect("candidate");
    candidate.representations.clear();
    candidate.source_requirements.clear();

    let bodies = burn_clear_laws(OperationId::Clear)
        .into_iter()
        .map(|law| law.relation_declaration())
        .filter(|row| {
            matches!(
                row.relation,
                Relation::ClearClamp
                    | Relation::ClearAssignment { .. }
                    | Relation::ClearFloor
                    | Relation::ClearFlow
                    | Relation::ClearBinding
                    | Relation::ClearCanonicalDeltaPolicy
            )
        })
        .collect::<Vec<_>>();
    let covered = bodies
        .iter()
        .map(|row| match row.relation {
            Relation::ClearClamp => 0,
            Relation::ClearAssignment { .. } => 1,
            Relation::ClearFloor => 2,
            Relation::ClearFlow => 3,
            Relation::ClearBinding => 4,
            Relation::ClearCanonicalDeltaPolicy => 5,
            _ => unreachable!("filtered to Clear bodies"),
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(covered, BTreeSet::from([0, 1, 2, 3, 4, 5]));

    for mut row in bodies {
        row.id = RelationId::new(
            OperationId::CompactAsh,
            row.id.kind(),
            row.id.subject().clone(),
        );
        row.proof_alternatives = BTreeSet::from([realization::ProofAlternativeId::new(
            row.id.clone(),
            ProofKind::ManifestShape,
        )]);
        let mut control = row.clone();
        control.relation = Relation::PermissionlessAuthorization;
        assert!(
            crate::coverage::relation_mutations(
                &control.relation,
                crate::case::ConservedAmountVisibility::Readable
            )
            .is_ok()
        );
        assert_eq!(
            crate::coverage::relation_mutations(
                &row.relation,
                crate::case::ConservedAmountVisibility::Readable
            ),
            Err(CompileError::UnsupportedRelationPlan)
        );
        assert!(relation_operands(&control).is_ok());
        assert_eq!(
            relation_operands(&row),
            Err(CompileError::UnsupportedRelationPlan)
        );

        let control_graph = crate::relation::build_relation_graph(
            &[OperationId::CompactAsh],
            std::slice::from_ref(&control),
            &[],
        )
        .expect("control graph");
        let cases =
            crate::case::execution_cases(&control_graph, &candidate).expect("control cases");
        assert_eq!(cases.len(), 1);
        assert_eq!(
            crate::case::validate_case_census(&control_graph, &candidate, &cases),
            Ok(())
        );
        let graph = crate::relation::build_relation_graph(
            &[OperationId::CompactAsh],
            std::slice::from_ref(&row),
            &[],
        )
        .expect("Clear body graph");
        assert_eq!(
            crate::case::validate_case_census(&graph, &candidate, &cases),
            Err(CompileError::UnsupportedRelationPlan)
        );
        for case in &cases {
            assert!(
                crate::placement::classify_relation_case(
                    &control,
                    case,
                    crate::sponsor_region::OrdinaryLbtcRole::Absent,
                )
                .is_ok()
            );
            assert_eq!(
                crate::placement::classify_relation_case(
                    &row,
                    case,
                    crate::sponsor_region::OrdinaryLbtcRole::Absent,
                ),
                Err(CompileError::UnsupportedRelationPlan)
            );
        }
        assert!(crate::layout::derive_layout_requirements(&control_graph, &[], &[]).is_ok());
        assert_eq!(
            crate::layout::derive_layout_requirements(&graph, &[], &[]),
            Err(CompileError::UnsupportedRelationPlan)
        );
        assert!(
            crate::engine_attestation::foundation::lower(&control_graph, &expressions)
                .validate()
                .is_ok()
        );
        assert_eq!(
            crate::engine_attestation::foundation::lower(&graph, &expressions).validate(),
            Err(CompileError::UnsupportedRelationPlan)
        );
        crate::engine_attestation::requirement::prepare_from_analyses(
            &control_graph,
            &construction,
        )
        .expect("supported control requirements");
        assert!(matches!(
            crate::engine_attestation::requirement::prepare_from_analyses(&graph, &construction),
            Err(CompileError::UnsupportedRelationPlan)
        ));
    }

    let clear_realization = realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::from_operations([OperationId::Clear]).expect("Clear scope"),
    )
    .expect("complete Clear realization");
    let clear_input = crate::input::bind_attestation_input(
        &architecture::ARCHITECTURE,
        clear_realization,
        crate::CompilationScope::from_operations([OperationId::Clear])
            .expect("compiler Clear scope"),
        super::test_policy(),
    )
    .expect("validated internal Clear input");
    assert!(matches!(
        crate::engine_attestation::source::lower(clear_input.owner()),
        Err(CompileError::UnsupportedRelationPlan)
    ));
}

#[test]
fn capability_views_prune_fail_closed() {
    let required = BTreeSet::from([
        RequiredCapability::ExactPublicAmountArithmetic,
        RequiredCapability::AuthenticatedObjectRecognition,
    ]);

    assert!(CapabilityView::Unconstrained.supports(&required));
    assert!(CapabilityView::Available(required.clone()).supports(&required));
    assert!(
        CapabilityView::Available(BTreeSet::from([
            RequiredCapability::ExactPublicAmountArithmetic,
            RequiredCapability::AuthenticatedObjectRecognition,
            RequiredCapability::OwnerAuthorization,
        ]))
        .supports(&required)
    );
    // A missing capability rejects; nothing is invented.
    assert!(
        !CapabilityView::Available(BTreeSet::from([
            RequiredCapability::ExactPublicAmountArithmetic,
        ]))
        .supports(&required)
    );
    assert!(!CapabilityView::Available(BTreeSet::new()).supports(&required));
}

// --- operand census over the real pilots (§17.2) ---

#[test]
fn every_pilot_relation_has_a_complete_sourced_operand_census() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let analysis = build_relation_analysis(&input).expect("relations");

    for node in analysis.project().nodes {
        let operands = relation_operands(&node.source).expect("operands derive");
        assert!(
            !operands.is_empty(),
            "relation {:?} has no operands",
            node.source.id
        );

        // Every approved proof alternative yields one authenticated
        // source row per operand, canonically sorted.
        for alternative in &node.source.proof_alternatives {
            let rows =
                derive_source_requirements(&node.source, alternative.proof()).expect("sources");

            assert_eq!(rows.len(), operands.len(), "{:?}", node.source.id);
            assert!(rows.is_sorted());

            for row in &rows {
                assert!(operands.contains(&row.operand));
                assert!(!is_sponsor_amount_operand(row.operand.role()));
            }
        }
    }
}

#[test]
fn conservation_proofs_classify_amount_sources_by_proof() {
    let conservation = declaration(
        OperationId::TransferLive,
        RelationKind::Conservation,
        Relation::AmountConservation {
            asset: architecture::AssetId::U,
            input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
            output_objects: BTreeSet::from([ObjectId::ReceiptLive]),
        },
    );

    // Public arithmetic requires public exact amounts…
    let public = derive_source_requirements(&conservation, ProofKind::PublicArithmetic).unwrap();
    assert!(public.iter().all(|row| {
        !matches!(row.operand.role(), OperandRole::ObjectFamilyAmount { .. })
            || row.source == RequiredSourceKind::AuthenticatedConsensusValue
    }));

    // …while confidential conservation requires commitment relations,
    // never exact public amounts.
    let confidential =
        derive_source_requirements(&conservation, ProofKind::ConfidentialConservation).unwrap();
    assert!(confidential.iter().all(|row| {
        !matches!(row.operand.role(), OperandRole::ObjectFamilyAmount { .. })
            || row.source == RequiredSourceKind::AuthenticatedCommitmentRelation
    }));
    assert!(
        confidential
            .iter()
            .all(|row| row.source != RequiredSourceKind::AuthenticatedConsensusValue)
    );
}

#[test]
fn authorization_sources_match_their_class() {
    // Owner authorization requires the owner census and an input-owner
    // witness of the exact family.
    let owner = declaration(
        OperationId::TransferLive,
        RelationKind::Authorization,
        Relation::OwnerAuthorization {
            object: ObjectId::ReceiptLive,
        },
    );
    let rows = derive_source_requirements(&owner, ProofKind::SignerMembership).unwrap();
    assert!(rows.iter().any(|row| {
        row.source == RequiredSourceKind::InputOwnerWitness
            && row.availability
                == AvailabilityClass::InputOwners {
                    object: ObjectId::ReceiptLive,
                }
    }));

    // Permissionless authorization requires no private witness at all.
    let permissionless = declaration(
        OperationId::CompactAsh,
        RelationKind::Authorization,
        Relation::PermissionlessAuthorization,
    );
    let rows =
        derive_source_requirements(&permissionless, ProofKind::PublicConstructibility).unwrap();
    assert!(
        rows.iter()
            .all(|row| row.availability == AvailabilityClass::Public),
        "permissionless authorization must not require a private witness",
    );
}

#[test]
fn sponsor_isolation_reads_no_sponsor_amount() {
    let sponsor = declaration(
        OperationId::CompactAsh,
        RelationKind::SponsorIsolation,
        Relation::SponsorIsolation,
    );
    let operands = relation_operands(&sponsor).unwrap();

    assert!(
        operands
            .iter()
            .all(|operand| { !matches!(operand.role(), OperandRole::ObjectFamilyAmount { .. }) })
    );

    // The sponsor witness stays sponsor-local and sponsor-activated.
    let rows = derive_source_requirements(&sponsor, ProofKind::ManifestShape).unwrap();
    let witness = rows
        .iter()
        .find(|row| row.source == RequiredSourceKind::SponsorLocalWitness)
        .expect("sponsor witness row");
    assert_eq!(witness.availability, AvailabilityClass::SponsorLocal);
    assert_eq!(
        witness.activation,
        RequirementActivation::WhenSponsorPresent
    );
}

#[test]
fn a_sponsor_amount_operand_is_structurally_rejected() {
    let poisoned = declaration(
        OperationId::CompactAsh,
        RelationKind::Conservation,
        Relation::AmountConservation {
            asset: architecture::AssetId::Lbtc,
            input_objects: BTreeSet::from([ObjectId::PlainLbtc]),
            output_objects: BTreeSet::from([ObjectId::PlainLbtc]),
        },
    );

    assert_eq!(
        relation_operands(&poisoned).unwrap_err(),
        CompileError::SponsorValueRead,
    );
}

#[test]
fn evidence_bounds_and_predicates_stay_typed() {
    // External substrate conservation remains a typed evidence operand.
    let substrate = declaration(
        OperationId::CompactAsh,
        RelationKind::SubstrateConservation,
        Relation::SubstrateConservation {
            asset: architecture::AssetId::Lbtc,
        },
    );
    let rows = derive_source_requirements(&substrate, ProofKind::SubstrateConservation).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, RequiredSourceKind::ExternalEvidence);
    assert!(matches!(
        rows[0].operand.role(),
        OperandRole::ExternalEvidence { .. }
    ));

    // Runtime bounds remain architecture-owned.
    let cardinality = declaration(
        OperationId::CompactAsh,
        RelationKind::Cardinality,
        Relation::Cardinality {
            side: realization::ObservedSide::Input,
            object: ObjectId::Ash,
            minimum: realization::Count::new(2),
            maximum: realization::CardinalityMaximum::Bound(architecture::BoundId::AshBatchMax),
        },
    );
    let rows = derive_source_requirements(&cardinality, ProofKind::ManifestShape).unwrap();
    assert!(rows.iter().any(|row| {
        row.source == RequiredSourceKind::RuntimeArchitectureBound
            && matches!(
                row.operand.role(),
                OperandRole::RuntimeBound {
                    bound: architecture::BoundId::AshBatchMax,
                }
            )
    }));

    // Expression predicates retain the exact source expression.
    let expression = realization::ExprId::relation(
        RelationId::new(
            OperationId::CompactAsh,
            RelationKind::CanonicalDeltaPolicy,
            RelationSubject::Operation,
        ),
        realization::ExpressionRole::Predicate,
    );
    let predicate = declaration(
        OperationId::CompactAsh,
        RelationKind::CanonicalDeltaPolicy,
        Relation::ExpressionPredicate {
            expression: expression.clone(),
        },
    );
    let operands = relation_operands(&predicate).unwrap();
    assert!(operands.iter().any(|operand| {
        matches!(
            operand.role(),
            OperandRole::ExpressionResult { expression: e } if *e == expression
        )
    }));
}

#[test]
fn operator_authorization_requires_an_external_evidence_operand() {
    let declaration = declaration(
        OperationId::AnnounceMaturity,
        RelationKind::Authorization,
        Relation::OperatorAuthorization,
    );
    assert_operator_source(&declaration);
    let operands = relation_operands(&declaration).unwrap();
    assert_eq!(operands.len(), 1);
    assert_eq!(
        operands[0].role(),
        &OperandRole::ExternalEvidence {
            requirement: realization::ExternalEvidenceRequirement::OperatorAuthorization {
                operation: OperationId::AnnounceMaturity
            }
        }
    );
}

#[test]
fn operator_constructibility_requires_an_external_evidence_operand() {
    let declaration = declaration(
        OperationId::AnnounceMaturity,
        RelationKind::Constructibility,
        Relation::Constructibility {
            class: realization::ConstructibilityClass::Operator,
        },
    );
    assert_operator_source(&declaration);
    let operands = relation_operands(&declaration).unwrap();
    assert_eq!(operands.len(), 1);
    assert_eq!(
        operands[0].role(),
        &OperandRole::ExternalEvidence {
            requirement: realization::ExternalEvidenceRequirement::OperatorAuthorization {
                operation: OperationId::AnnounceMaturity
            }
        }
    );
}

fn assert_operator_source(declaration: &RelationDeclaration) {
    assert_eq!(
        crate::source::proof_capabilities(declaration, ProofKind::ManifestShape),
        BTreeSet::from([
            RequiredCapability::AuthenticatedObjectRecognition,
            RequiredCapability::OperatorAuthorization
        ]),
    );
    assert_eq!(
        derive_source_requirements(declaration, ProofKind::ManifestShape).unwrap(),
        vec![crate::source::SourceRequirement {
            operand: crate::source::OperandId::new(
                declaration.id.clone(),
                OperandRole::ExternalEvidence {
                    requirement: realization::ExternalEvidenceRequirement::OperatorAuthorization {
                        operation: OperationId::AnnounceMaturity
                    },
                }
            ),
            source: RequiredSourceKind::ExternalEvidence,
            availability: AvailabilityClass::Public,
            activation: RequirementActivation::Always,
        }]
    );
}

fn announcement_retained_operands(id: &RelationId) -> Vec<OperandRole> {
    use OperandRole::{
        CanonicalPartition, ExternalEvidence, ObjectFamilyCount, ObjectFamilyMembers, OpenFlowSet,
        ProjectionSet, RootEffects, RuntimeBound, SponsorRegion, SponsorSignerSet,
    };
    use RelationKind::{
        AllowedObjectFamilies, Authorization, CanonicalDeltaPolicy, Cardinality, Constructibility,
        Lifecycle, OpenFlowPolicy, ProjectionPolicy, Recognition, RootPolicy,
        SponsorEnvelopeMultiplicity, SponsorIsolation, SubstrateConservation,
    };
    use architecture::{AssetId, BoundId};
    use realization::{ExternalEvidenceRequirement as Evidence, TransactionSide::Input};
    let operation = AnnounceMaturity;
    match (id.kind(), id.subject()) {
        (Cardinality | Recognition, RelationSubject::ObjectFamily { side, object }) => {
            assert!([State, PlainLbtc].contains(object));
            let (side, object) = (*side, *object);
            let mut roles = if id.kind() == Cardinality {
                vec![ObjectFamilyCount { side, object }]
            } else {
                vec![ObjectFamilyMembers { side, object }]
            };
            if (id.kind(), side, object) == (Cardinality, Input, PlainLbtc) {
                roles.push(RuntimeBound {
                    bound: BoundId::FeeSponsorInputMax,
                });
            }
            roles
        }
        (AllowedObjectFamilies, RelationSubject::TransactionSide { side }) => [State, PlainLbtc]
            .map(|object| ObjectFamilyMembers {
                side: *side,
                object,
            })
            .into(),
        (Authorization | Constructibility, RelationSubject::Operation) => vec![ExternalEvidence {
            requirement: Evidence::OperatorAuthorization { operation },
        }],
        (
            SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        ) => vec![ExternalEvidence {
            requirement: Evidence::SubstrateConservation {
                operation,
                asset: AssetId::Lbtc,
            },
        }],
        (SponsorIsolation, RelationSubject::Sponsor) => vec![SponsorRegion, SponsorSignerSet],
        (SponsorEnvelopeMultiplicity, RelationSubject::Sponsor)
        | (OpenFlowPolicy, RelationSubject::Operation) => vec![OpenFlowSet],
        (CanonicalDeltaPolicy, RelationSubject::Operation) => vec![CanonicalPartition],
        (RootPolicy, RelationSubject::Operation) => vec![RootEffects],
        (ProjectionPolicy, RelationSubject::Operation) => vec![ProjectionSet],
        (RelationKind::Representation, RelationSubject::Representation { object: State })
        | (Lifecycle, RelationSubject::LifecycleExit { object: State, .. }) => vec![],
        other => panic!("unexpected announcement relation {other:?}"),
    }
}

#[test]
fn announcement_sources_pin_retention_and_vacuous_bound_erasure() {
    use crate::{
        analyzed_operation::analyze_operation,
        case::{SponsorCase, execution_cases},
        constructibility::build_constructibility_analysis,
        placement::{DischargeBoundary, PlacementSearchLimits},
        proof::enumerate_feasible_plans,
        requirement::derive_relation_requirements,
    };
    let input = announcement_input();
    let relations = build_relation_analysis(&input).unwrap();
    let construction = build_constructibility_analysis(&input).unwrap();
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).unwrap();
    for candidate in plans.candidates {
        assert_eq!(candidate.source_requirements.len(), 23);
        let owned = derive_relation_requirements(&relations, &construction, &candidate).unwrap();
        assert_eq!(owned.len(), 26);
        for (id, rows) in &owned {
            let actual: BTreeSet<_> = rows
                .source_requirements
                .iter()
                .map(|row| row.operand.role().clone())
                .collect();
            assert_eq!(
                actual,
                announcement_retained_operands(id).into_iter().collect(),
                "{id:?}"
            );
        }
        let cases = execution_cases(&relations, &candidate).unwrap();
        let absent = cases
            .iter()
            .find(|case| case.id.sponsor == SponsorCase::Absent)
            .unwrap();
        assert_eq!(absent.active_sources.len(), 15);
        let bound = absent
            .active_sources
            .iter()
            .find(|row| matches!(row.operand.role(), OperandRole::RuntimeBound { .. }))
            .unwrap();
        assert_eq!(bound.source, RequiredSourceKind::RuntimeArchitectureBound);
        assert_eq!(bound.activation, RequirementActivation::Always);
        let limits = PlacementSearchLimits::new(
            10_000_000_u64.try_into().unwrap(),
            1_000_000_u64.try_into().unwrap(),
        );
        let analyzed =
            analyze_operation(&relations, &owned, &candidate, AnnounceMaturity, limits).unwrap();
        let rows: Vec<_> = analyzed
            .relation_cases
            .values()
            .filter(|row| row.key.case.sponsor == SponsorCase::Absent)
            .collect();
        assert_eq!(
            rows.iter()
                .map(|row| row.active_sources.len())
                .sum::<usize>(),
            14
        );
        assert_eq!(
            rows.iter()
                .filter(|row| row.boundaries.contains(&DischargeBoundary::RuntimeCarrier))
                .map(|row| row.active_sources.len())
                .sum::<usize>(),
            11
        );
        assert!(rows.iter().all(|row| !row.active_sources.contains(bound)));
    }
}
