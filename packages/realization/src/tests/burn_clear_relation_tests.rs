use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    AssetId, BoundId, DeltaKind, ObjectId, OperationId, ProjectionId, RootId, TagId,
};

use crate::burn_clear_laws::*;
use crate::validate::{validate_operation_ownership, validate_relation_identity};
use crate::*;

fn amount(value: u64) -> ProtocolAmount {
    ProtocolAmount::new(value).unwrap()
}

#[test]
fn burn_law_edges_have_literal_endpoints_and_refuse_omission_or_change() {
    let input = RelationId::new(
        OperationId::Burn,
        RelationKind::SideRepresentation,
        RelationSubject::SideRepresentation {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    );
    let binding = RelationId::new(
        OperationId::Burn,
        RelationKind::BurnBinding,
        RelationSubject::Operation,
    );
    let data = RelationId::new(
        OperationId::Burn,
        RelationKind::DataOutputs,
        RelationSubject::Operation,
    );
    let expected = vec![
        RelationDependencyDeclaration {
            prerequisite: input,
            dependent: binding.clone(),
            edge: RelationEdge::StaticRequirement,
        },
        RelationDependencyDeclaration {
            prerequisite: binding,
            dependent: data,
            edge: RelationEdge::StaticRequirement,
        },
    ];
    assert_eq!(burn_law_dependencies(), expected);
    validate_burn_law_dependencies(&expected).unwrap();
    for index in 0..2 {
        let mut omitted = expected.clone();
        omitted.remove(index);
        let mut changed = expected.clone();
        changed[index].edge = RelationEdge::RecognitionBeforeValue;
        let mut duplicate = expected.clone();
        duplicate.push(expected[index].clone());
        for subject in [omitted, changed, duplicate] {
            assert_eq!(
                validate_burn_law_dependencies(&subject),
                Err(RealizationError::ArchitectureRelationMismatch {
                    relation: RelationId::new(
                        OperationId::Burn,
                        RelationKind::BurnBinding,
                        RelationSubject::Operation
                    ),
                })
            );
        }
    }
}

// Synthetic graph under the public evaluator: complete declaration dispatch
// remains separate from this law-level activation fixture.
#[allow(clippy::too_many_lines)] // Keep the synthetic law and dependency census together.
fn activated_declaration(operation: OperationId) -> OperationRealization {
    let laws = burn_clear_laws(operation);
    let rows = laws
        .iter()
        .map(BurnClearLawDeclaration::relation_declaration)
        .collect::<Vec<_>>();
    let mut expressions = Vec::new();
    if operation == OperationId::Clear {
        for field in StateField::ALL {
            for side in [TransactionSide::Input, TransactionSide::Output] {
                let fact = FactId::StateField {
                    operation,
                    side,
                    field: *field,
                };
                expressions.push(ExpressionDeclaration {
                    id: ExprId::fact(fact.clone()),
                    ty: fact.semantic_type(),
                    node: ExpressionNode::Fact(fact),
                });
            }
        }
        for row in &rows {
            if let Relation::ClearAssignment {
                field,
                expression: Some(expression),
            } = &row.relation
            {
                expressions.push(ExpressionDeclaration {
                    id: expression.clone(),
                    ty: SemanticType::Bool,
                    node: copy_expression(*field),
                });
            }
        }
    }
    let mut dependencies = Vec::new();
    if operation == OperationId::Clear {
        let clamp = RelationId::new(
            operation,
            RelationKind::ClearClamp,
            RelationSubject::Operation,
        );
        let floor = RelationId::new(
            operation,
            RelationKind::ClearFloor,
            RelationSubject::Operation,
        );
        let binding = RelationId::new(
            operation,
            RelationKind::ClearBinding,
            RelationSubject::Operation,
        );
        for row in &rows {
            if matches!(
                row.id.kind(),
                RelationKind::ClearAssignment
                    | RelationKind::ClearFlow
                    | RelationKind::DataOutputs
                    | RelationKind::CanonicalDeltaPolicy
            ) {
                dependencies.push(RelationDependencyDeclaration {
                    prerequisite: clamp.clone(),
                    dependent: row.id.clone(),
                    edge: RelationEdge::StaticRequirement,
                });
            }
            if row.id.kind() == RelationKind::ClearAssignment {
                dependencies.push(RelationDependencyDeclaration {
                    prerequisite: row.id.clone(),
                    dependent: floor.clone(),
                    edge: RelationEdge::StaticRequirement,
                });
            }
            if matches!(
                row.id.kind(),
                RelationKind::ClearFloor | RelationKind::ClearFlow | RelationKind::DataOutputs
            ) {
                dependencies.push(RelationDependencyDeclaration {
                    prerequisite: row.id.clone(),
                    dependent: binding.clone(),
                    edge: RelationEdge::StaticRequirement,
                });
            }
        }
    }
    if operation == OperationId::Burn {
        dependencies.extend(burn_law_dependencies());
    }
    OperationRealization {
        operation,
        expressions,
        relations: rows,
        relation_dependencies: dependencies,
        constructibility_nodes: Vec::new(),
        constructibility_edges: Vec::new(),
        lifecycle_nodes: Vec::new(),
        lifecycle_edges: Vec::new(),
        disclosure_nodes: Vec::new(),
        disclosure_edges: Vec::new(),
        disclosure_seeds: Vec::new(),
    }
}

fn activated_graph(operation: OperationId) -> ScopedRealizationSpec {
    let declaration = activated_declaration(operation);
    let mut realized = derive(
        &architecture::ARCHITECTURE,
        RealizationScope::phase1_pilots(),
    )
    .unwrap();
    (
        realized.relation_graph,
        realized.relation_node_by_id,
        realized.relation_evaluation_order,
    ) = crate::relation::build_relation_graph(
        declaration.relations.clone(),
        declaration.relation_dependencies.clone(),
    )
    .unwrap();
    (
        realized.expression_graph,
        realized.expression_node_by_id,
        realized.expression_evaluation_order,
    ) = crate::expression::build_expression_graph(declaration.expressions.clone()).unwrap();
    realized.scope = RealizationScope::from_operations([operation]).unwrap();
    realized.operations = BTreeMap::from([(operation, declaration)]);
    realized
}

#[test]
fn public_copy_expression_validation_refuses_changed_operands() {
    let control = activated_declaration(OperationId::Clear);
    assert_eq!(control.relations.len(), 12);
    assert_eq!(control.expressions.len(), 17);
    assert_eq!(
        validate_operation_ownership(OperationId::Clear, &control),
        Ok(())
    );
    for row in &control.relations {
        let Relation::ClearAssignment {
            expression: Some(expression),
            ..
        } = &row.relation
        else {
            continue;
        };
        let mut subject = control.clone();
        let node = subject
            .expressions
            .iter_mut()
            .find(|node| node.id == *expression)
            .unwrap();
        let ExpressionNode::Equal { left, right } = &mut node.node else {
            panic!("copy expression must be Equal");
        };
        // Only the right operand's side changes from Output to Input.
        *right = left.clone();
        assert_eq!(
            validate_operation_ownership(OperationId::Clear, &subject),
            Err(RealizationError::ArchitectureRelationMismatch {
                relation: row.id.clone()
            }),
        );
    }
}

#[test]
fn public_bound_entry_refuses_undeclared_operations() {
    for (common, extension) in [burn(), clear(10, 6, 3, 6, 0, 4)] {
        let bound = checked(&common, &extension);
        let control = activated_graph(common.operation);
        assert!(
            control
                .evaluate_operation_with_burn_clear(&bound)
                .unwrap()
                .is_conformant()
        );
        let mut subject = control;
        subject.operations.remove(&common.operation);
        assert_eq!(
            subject.evaluate_operation_with_burn_clear(&bound),
            Err(RealizationError::OperationOutsideScope(common.operation))
        );
        let pilot = derive(
            &architecture::ARCHITECTURE,
            RealizationScope::phase1_pilots(),
        )
        .unwrap();
        assert_eq!(
            pilot.evaluate_operation_with_burn_clear(&bound),
            Err(RealizationError::OperationOutsideScope(common.operation))
        );
    }
}

#[test]
fn activated_public_laws_accept_controls_and_require_the_extension() {
    for (common, extension) in [burn(), clear(10, 6, 3, 6, 0, 4), clear(2, 6, 3, 2, 4, 0)] {
        let realized = activated_graph(common.operation);
        let report = realized
            .evaluate_operation_with_burn_clear(&checked(&common, &extension))
            .unwrap();
        assert_eq!(
            report.verdicts.len(),
            if common.operation == OperationId::Burn {
                5
            } else {
                12
            }
        );
        assert!(
            report
                .verdicts
                .iter()
                .all(|row| row.status == RelationStatus::Passed)
        );
        assert_eq!(
            realized.evaluate_operation(&common),
            Err(RealizationError::BurnClearObservation(
                BurnClearRefusal::MissingExtension
            ))
        );
    }
}

#[test]
fn failed_public_clamp_blocks_dependents_instead_of_aborting_the_report() {
    for (common, mut extension) in [clear(10, 6, 3, 6, 0, 4), clear(10, 2, 0, 1, 1, 9)] {
        let realized = activated_graph(OperationId::Clear);
        assert!(
            realized
                .evaluate_operation_with_burn_clear(&checked(&common, &extension))
                .unwrap()
                .is_conformant()
        );
        if let BurnClearPayload::Clear { predecessor, .. } = &mut extension.payload {
            predecessor.metadata.y_l = amount(0);
        }
        let report = realized
            .evaluate_operation_with_burn_clear(&checked(&common, &extension))
            .unwrap();
        let clamp = RelationId::new(
            OperationId::Clear,
            RelationKind::ClearClamp,
            RelationSubject::Operation,
        );
        assert_eq!(
            report.verdict(&clamp).unwrap().status,
            RelationStatus::Failed {
                reason: RelationFailure::BurnClearLaw
            }
        );
        for row in report.verdicts.iter().filter(|row| row.relation != clamp) {
            assert!(matches!(row.status, RelationStatus::Blocked { .. }));
        }
        let delta = RelationId::new(
            OperationId::Clear,
            RelationKind::CanonicalDeltaPolicy,
            RelationSubject::Operation,
        );
        assert_eq!(
            report.verdict(&delta).unwrap().status,
            RelationStatus::Blocked {
                prerequisites: vec![clamp]
            }
        );
    }
}

#[test]
fn activated_census_checks_every_identity_body_and_multiplicity() {
    for (operation, count) in [(OperationId::Burn, 5), (OperationId::Clear, 12)] {
        let rows = burn_clear_laws(operation)
            .iter()
            .map(BurnClearLawDeclaration::relation_declaration)
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), count);
        assert_eq!(
            validate_burn_clear_relation_census(operation, &rows),
            Ok(())
        );
        for (index, row) in rows.iter().enumerate() {
            validate_relation_identity(row).unwrap();
            let mut missing = rows.clone();
            missing.remove(index);
            assert_eq!(
                validate_burn_clear_relation_census(operation, &missing),
                Err(BurnClearLawError::Census)
            );
            let mut duplicate = rows.clone();
            duplicate.push(row.clone());
            assert_eq!(
                validate_burn_clear_relation_census(operation, &duplicate),
                Err(BurnClearLawError::Census)
            );
            let mut foreign = row.clone();
            foreign.id = RelationId::new(
                OperationId::CompactAsh,
                row.id.kind(),
                row.id.subject().clone(),
            );
            if requires_extension(&row.relation) {
                assert_eq!(
                    validate_relation_identity(&foreign),
                    Err(RealizationError::ArchitectureRelationMismatch {
                        relation: foreign.id.clone()
                    })
                );
            }
        }
        let mut reversed = rows;
        reversed.reverse();
        assert_eq!(
            validate_burn_clear_relation_census(operation, &reversed),
            Ok(())
        );
    }
}
fn reference(side: ObservedSide, ordinal: u32) -> ObservedObjectRef {
    ObservedObjectRef { side, ordinal }
}
fn object(side: ObservedSide, ordinal: u32, kind: ObjectId, value: u64) -> ObservedObject {
    ObservedObject {
        reference: reference(side, ordinal),
        kind: ObservedObjectKind::Declared(kind),
        asset: ObservedAsset::Declared(if kind == ObjectId::State {
            AssetId::Pid
        } else {
            AssetId::U
        }),
        value: ObservedValue::Protocol(amount(value)),
        owner: (kind == ObjectId::ReceiptLive).then_some(OwnerId([1; 32])),
        representation: RepresentationMode::Explicit,
    }
}
fn context(common: &OperationObservation) -> BurnClearContext {
    BurnClearContext::bind_observation(common.clone(), [1; 32], [2; 32]).unwrap()
}
fn common(operation: OperationId, objects: Vec<ObservedObject>) -> OperationObservation {
    OperationObservation {
        operation,
        region: ObservedOperationRegion {
            members: objects.iter().map(|object| object.reference).collect(),
            ..Default::default()
        },
        objects,
        other_regions: vec![],
        protocol_signers: BTreeSet::from([OwnerId([1; 32])]),
        sponsor_signers: BTreeSet::new(),
        root_effects: vec![],
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::AshBatchMax, Count::new(4)),
            (BoundId::BurnRecordMax, Count::new(4)),
        ]),
    }
}
fn checked(
    common: &OperationObservation,
    extension: &BurnClearObservation,
) -> BoundBurnClearObservation {
    validate_burn_clear_observation(common, &context(common), Some(extension)).unwrap()
}
fn law(operation: OperationId, key: BurnClearLawKey) -> BurnClearLawDeclaration {
    burn_clear_laws(operation)
        .into_iter()
        .find(|law| law.id.key == key)
        .unwrap()
}
fn holds(
    key: BurnClearLawKey,
    common: &OperationObservation,
    extension: &BurnClearObservation,
) -> bool {
    let declaration = law(common.operation, key).relation_declaration();
    let id = declaration.id.clone();
    let mut realized = activated_graph(common.operation);
    (
        realized.relation_graph,
        realized.relation_node_by_id,
        realized.relation_evaluation_order,
    ) = crate::relation::build_relation_graph([declaration], []).unwrap();
    let report = realized
        .evaluate_operation_with_burn_clear(&checked(common, extension))
        .unwrap();
    report.verdict(&id).unwrap().status == RelationStatus::Passed
}

pub(super) fn burn() -> (OperationObservation, BurnClearObservation) {
    let mut common = common(
        OperationId::Burn,
        vec![
            object(ObservedSide::Input, 0, ObjectId::ReceiptLive, 12),
            object(ObservedSide::Output, 0, ObjectId::Ash, 10),
            object(ObservedSide::Output, 1, ObjectId::ReceiptLive, 2),
        ],
    );
    common.objects[2].representation = RepresentationMode::PrivateCommitted;
    common.projections.insert(ProjectionId::BurnEvent);
    common
        .region
        .canonical_partition
        .flows
        .push(ObservedCanonicalFlow {
            asset: AssetId::U,
            sources: vec![reference(ObservedSide::Input, 0)],
            destinations: vec![
                reference(ObservedSide::Output, 0),
                reference(ObservedSide::Output, 1),
            ],
            movement_kind: Some(DeltaKind::Lateral),
            destructions: vec![],
        });
    let fresh_ash = reference(ObservedSide::Output, 0);
    let extension = BurnClearObservation {
        context: context(&common),
        payload: BurnClearPayload::Burn {
            fresh_ash,
            records: vec![BurnRecord {
                ordinal: 0,
                address: BurnAddress([9; 32]),
                amount: amount(13),
                tag: TagId::Burn,
                fresh_ash,
            }],
        },
    };
    (common, extension)
}

// Literal test inputs and expected successor: no expected value uses the law.
fn clear(
    b: u64,
    yl: u64,
    yt: u64,
    x: u64,
    next_yl: u64,
    residual: u64,
) -> (OperationObservation, BurnClearObservation) {
    let mut objects = vec![
        object(ObservedSide::Input, 0, ObjectId::State, 1),
        object(ObservedSide::Input, 1, ObjectId::Ash, b),
        object(ObservedSide::Output, 0, ObjectId::State, 1),
    ];
    if residual != 0 {
        objects.push(object(ObservedSide::Output, 1, ObjectId::Ash, residual));
    }
    let mut common = common(OperationId::Clear, objects);
    common.protocol_signers.clear();
    common.projections.insert(ProjectionId::ClearEvent);
    common.root_effects.push(ObservedRootEffect {
        root: RootId::State,
        effect: ObservedRootEffectKind::Succession,
    });
    common
        .region
        .canonical_partition
        .flows
        .push(ObservedCanonicalFlow {
            asset: AssetId::U,
            sources: vec![reference(ObservedSide::Input, 1)],
            destinations: if residual == 0 {
                vec![]
            } else {
                vec![reference(ObservedSide::Output, 1)]
            },
            movement_kind: (residual != 0).then_some(DeltaKind::OwnerlessLateral),
            destructions: vec![ObservedDestructionLeg {
                tag: TagId::Recon,
                amount: amount(x),
            }],
        });
    let predecessor = ObservedState {
        reference: reference(ObservedSide::Input, 0),
        metadata: StateMetadata {
            omega: amount(20),
            y_l: amount(yl),
            y_t: amount(yt),
            q: amount(2),
            cycle: Cycle::new(7),
            maturity: Maturity::Unannounced,
        },
    };
    let successor = ObservedState {
        reference: reference(ObservedSide::Output, 0),
        metadata: StateMetadata {
            y_l: amount(next_yl),
            ..predecessor.metadata
        },
    };
    let extension = BurnClearObservation {
        context: context(&common),
        payload: BurnClearPayload::Clear {
            predecessor,
            successor,
            ash_sources: vec![reference(ObservedSide::Input, 1)],
            destruction: Destruction {
                asset: AssetId::U,
                amount: amount(x),
                tag: TagId::Recon,
                flow: Some(0),
            },
        },
    };
    (common, extension)
}

#[test]
fn literal_census_identity_and_completeness() {
    use BurnClearLawKey as K;
    let burn_keys = vec![
        K::CanonicalDeltaPolicy,
        K::SideRepresentation(TransactionSide::Input),
        K::SideRepresentation(TransactionSide::Output),
        K::DataOutputs,
        K::BurnBinding,
    ];
    let mut clear_keys = vec![K::CanonicalDeltaPolicy, K::DataOutputs, K::ClearClamp];
    clear_keys.extend(StateField::ALL.iter().copied().map(K::ClearAssignment));
    clear_keys.extend([K::ClearFloor, K::ClearFlow, K::ClearBinding]);
    for (operation, keys) in [
        (OperationId::Burn, burn_keys),
        (OperationId::Clear, clear_keys),
    ] {
        let declarations = burn_clear_laws(operation);
        assert_eq!(
            declarations
                .iter()
                .map(|law| law.id.key)
                .collect::<Vec<_>>(),
            keys
        );
        validate_burn_clear_law_census(operation, &declarations).unwrap();
        let mut reversed = declarations.clone();
        reversed.reverse();
        validate_burn_clear_law_census(operation, &reversed).unwrap();
        for index in 0..declarations.len() {
            let mut missing = declarations.clone();
            missing.remove(index);
            assert_eq!(
                validate_burn_clear_law_census(operation, &missing),
                Err(BurnClearLawError::Census)
            );
            let mut duplicate = declarations.clone();
            duplicate.push(declarations[index].clone());
            assert_eq!(
                validate_burn_clear_law_census(operation, &duplicate),
                Err(BurnClearLawError::Census)
            );
            let mut wrong = declarations.clone();
            wrong[index].id.operation = OperationId::CompactAsh;
            assert_eq!(
                validate_burn_clear_law_census(operation, &wrong),
                Err(BurnClearLawError::Census)
            );
        }
    }
}

#[test]
fn every_staged_law_accepts_its_control() {
    for (common, extension) in [
        burn(),
        clear(10, 6, 3, 6, 0, 4),
        clear(4, 6, 3, 4, 2, 0),
        clear(10, 6, 0, 5, 1, 5),
        clear(6, 6, 3, 6, 0, 0),
        clear(5, 6, 0, 5, 1, 0),
        clear(10, 6, 1, 6, 0, 4),
        clear(6, 6, 1, 6, 0, 0),
    ] {
        let bound = checked(&common, &extension);
        for declaration in burn_clear_laws(common.operation) {
            assert_eq!(
                declaration.evaluate(&bound),
                Ok(true),
                "{:?}",
                declaration.id
            );
            let mut wrong = declaration;
            wrong.id.operation = OperationId::CompactAsh;
            assert_eq!(wrong.evaluate(&bound), Err(BurnClearLawError::Identity));
        }
    }
}

#[test]
fn each_assignment_uses_the_carried_field_and_its_owned_equal() {
    for field in StateField::ALL {
        let (common, mut extension) = clear(10, 6, 3, 6, 0, 4);
        let key = BurnClearLawKey::ClearAssignment(*field);
        assert!(holds(key, &common, &extension));
        if let BurnClearPayload::Clear { successor, .. } = &mut extension.payload {
            match field {
                StateField::Omega => successor.metadata.omega = amount(21),
                StateField::YL => successor.metadata.y_l = amount(1),
                StateField::YT => successor.metadata.y_t = amount(4),
                StateField::Q => successor.metadata.q = amount(3),
                StateField::Cycle => successor.metadata.cycle = Cycle::new(8),
                StateField::Maturity => successor.metadata.maturity = Maturity::Complete,
            }
        }
        assert!(!holds(key, &common, &extension));
        let mut wrong = law(OperationId::Clear, key);
        if let BurnClearLaw::ClearAssignment { expression, .. } = &mut wrong.body {
            *expression = Some(ExpressionNode::Bool(true));
        }
        assert_eq!(
            wrong.evaluate(&checked(&common, &extension)),
            Err(BurnClearLawError::Identity)
        );
    }
}

#[test]
fn side_admission_cannot_be_widened_by_change_modes() {
    for side in [TransactionSide::Input, TransactionSide::Output] {
        let (mut common, mut extension) = burn();
        let key = BurnClearLawKey::SideRepresentation(side);
        assert!(holds(key, &common, &extension));
        match side {
            TransactionSide::Input => {
                common.objects[0].representation = RepresentationMode::PrivateCommitted;
            }
            TransactionSide::Output => {
                common.objects[2].representation = RepresentationMode::PublicCommitted;
            }
        }
        extension.context = context(&common);
        assert!(!holds(key, &common, &extension));
    }
}

#[test]
fn burn_binding_and_delta_disagreements_do_not_read_credit() {
    let (mut common, mut extension) = burn();
    assert!(holds(BurnClearLawKey::BurnBinding, &common, &extension));
    assert_eq!(
        evaluate_record_credit(&checked(&common, &extension)).unwrap(),
        RecordCredit::Overclaim {
            total: 13,
            fresh_amount: amount(10)
        }
    );
    common.protocol_signers.clear();
    extension.context = context(&common);
    assert!(!holds(BurnClearLawKey::BurnBinding, &common, &extension));
    let (mut common, mut extension) = burn();
    assert!(holds(
        BurnClearLawKey::CanonicalDeltaPolicy,
        &common,
        &extension
    ));
    common.region.canonical_partition.flows[0].movement_kind = Some(DeltaKind::OwnerlessLateral);
    extension.context = context(&common);
    assert!(!holds(
        BurnClearLawKey::CanonicalDeltaPolicy,
        &common,
        &extension
    ));
}

#[test]
fn conditional_delta_controls_and_focused_disagreements() {
    for fixture in [clear(10, 6, 3, 6, 0, 4), clear(4, 6, 3, 4, 2, 0)] {
        let (mut common, mut extension) = fixture;
        assert!(holds(
            BurnClearLawKey::CanonicalDeltaPolicy,
            &common,
            &extension
        ));
        common.region.canonical_partition.flows[0].movement_kind = if common.objects.len() == 4 {
            None
        } else {
            Some(DeltaKind::OwnerlessLateral)
        };
        extension.context = context(&common);
        assert!(!holds(
            BurnClearLawKey::CanonicalDeltaPolicy,
            &common,
            &extension
        ));
        assert!(!holds(BurnClearLawKey::ClearFlow, &common, &extension));
    }
}

#[test]
fn clear_floor_is_independent_of_clamp_and_copy() {
    for coordinate in 0..3 {
        let (common, mut extension) = clear(10, 6, 3, 6, 0, 4);
        assert!(holds(BurnClearLawKey::ClearFloor, &common, &extension));
        if let BurnClearPayload::Clear { successor, .. } = &mut extension.payload {
            match coordinate {
                0 => successor.metadata.y_t = amount(0),
                1 => successor.metadata.y_t = amount(10),
                _ => successor.metadata.omega = amount(21),
            }
        }
        assert!(!holds(BurnClearLawKey::ClearFloor, &common, &extension));
        assert!(holds(BurnClearLawKey::ClearClamp, &common, &extension));
    }
}

#[test]
fn binding_data_and_flow_have_distinct_disagreements() {
    let (mut common, mut extension) = clear(10, 6, 3, 6, 0, 4);
    assert!(holds(BurnClearLawKey::ClearBinding, &common, &extension));
    common.root_effects[0].effect = ObservedRootEffectKind::Termination;
    extension.context = context(&common);
    assert!(!holds(BurnClearLawKey::ClearBinding, &common, &extension));
    let (mut common, mut extension) = clear(10, 6, 3, 6, 0, 4);
    assert!(holds(BurnClearLawKey::DataOutputs, &common, &extension));
    common.region.canonical_partition.flows[0].destructions[0].amount = amount(5);
    if let BurnClearPayload::Clear { destruction, .. } = &mut extension.payload {
        destruction.amount = amount(5);
    }
    extension.context = context(&common);
    assert!(!holds(BurnClearLawKey::DataOutputs, &common, &extension));
    let (mut common, mut extension) = clear(10, 6, 3, 6, 0, 4);
    assert!(holds(BurnClearLawKey::ClearFlow, &common, &extension));
    common.objects[3].value = ObservedValue::Protocol(amount(5));
    extension.context = context(&common);
    assert!(!holds(BurnClearLawKey::ClearFlow, &common, &extension));
}

#[test]
fn structural_binding_refuses_duplicate_recon_wrong_association_and_missing_facts() {
    let (mut common, mut extension) = clear(10, 6, 3, 6, 0, 4);
    checked(&common, &extension);
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), None),
        Err(BurnClearRefusal::MissingExtension)
    );
    common.region.canonical_partition.flows[0].destructions = vec![
        ObservedDestructionLeg {
            tag: TagId::Recon,
            amount: amount(3)
        };
        2
    ];
    assert_eq!(
        validate_burn_clear_observation(&common, &extension.context, Some(&extension)),
        Err(BurnClearRefusal::MalformedObservation)
    );
    // Duplicate tags also refuse with unequal amounts and the same total.
    common.region.canonical_partition.flows[0].destructions[0].amount = amount(2);
    common.region.canonical_partition.flows[0].destructions[1].amount = amount(4);
    assert_eq!(
        validate_burn_clear_observation(&common, &extension.context, Some(&extension)),
        Err(BurnClearRefusal::MalformedObservation)
    );
    // A different tag reaches the exact single-leg binding check instead.
    common.region.canonical_partition.flows[0].destructions[1].tag = TagId::Burn;
    extension.context = context(&common);
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
        Err(BurnClearRefusal::DestructionBinding)
    );
    let (common, mut extension) = burn();
    checked(&common, &extension);
    if let BurnClearPayload::Burn { records, .. } = &mut extension.payload {
        records[0].fresh_ash.ordinal = 1;
    }
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
        Err(BurnClearRefusal::DataAssociation)
    );
    let (mut common, mut extension) = clear(10, 6, 3, 6, 0, 4);
    assert!(holds(BurnClearLawKey::ClearClamp, &common, &extension));
    common.bounds.remove(&BoundId::AshBatchMax);
    extension.context = context(&common);
    assert_eq!(
        law(OperationId::Clear, BurnClearLawKey::ClearClamp)
            .evaluate(&checked(&common, &extension)),
        Err(BurnClearLawError::Operand(
            BurnClearRefusal::ClearTransition
        ))
    );
    assert_eq!(
        activated_graph(OperationId::Clear)
            .evaluate_operation_with_burn_clear(&checked(&common, &extension)),
        Err(RealizationError::BurnClearObservation(
            BurnClearRefusal::ClearTransition
        ))
    );
}

#[test]
fn clamp_progress_failures_are_law_disagreements() {
    // B=10: each failed subject is one predecessor YL away from a control.
    for (yl, yt, expected) in [(6, 3, true), (0, 3, false), (2, 0, true), (0, 0, false)] {
        let (common, mut extension) = clear(10, 6, 3, 6, 0, 4);
        if let BurnClearPayload::Clear { predecessor, .. } = &mut extension.payload {
            predecessor.metadata.y_l = amount(yl);
            predecessor.metadata.y_t = amount(yt);
        }
        assert_eq!(
            holds(BurnClearLawKey::ClearClamp, &common, &extension),
            expected
        );
    }
}

#[test]
fn staging_refusal_names_and_proof_families_are_closed() {
    let errors = [
        BurnClearLawError::Identity,
        BurnClearLawError::Census,
        BurnClearLawError::Operand(BurnClearRefusal::MissingExtension),
        BurnClearLawError::Evaluation(RealizationError::CountOverflow),
    ];
    assert_eq!(
        errors
            .iter()
            .map(BurnClearLawError::name)
            .collect::<Vec<_>>(),
        ["identity", "census", "operand", "evaluation"]
    );
    for declaration in burn_clear_laws(OperationId::Clear)
        .into_iter()
        .chain(burn_clear_laws(OperationId::Burn))
    {
        let expected = match declaration.id.key {
            BurnClearLawKey::CanonicalDeltaPolicy
            | BurnClearLawKey::DataOutputs
            | BurnClearLawKey::BurnBinding
            | BurnClearLawKey::ClearBinding => Some(ProofKind::ManifestShape),
            BurnClearLawKey::SideRepresentation(_) => None,
            _ => Some(ProofKind::PublicArithmetic),
        };
        assert_eq!(declaration.body.proof_kind(), expected);
    }
}

#[test]
fn changed_body_refuses_census_and_evaluation() {
    let (common, extension) = burn();
    let mut declarations = burn_clear_laws(OperationId::Burn);
    validate_burn_clear_law_census(OperationId::Burn, &declarations).unwrap();
    let input = declarations
        .iter_mut()
        .find(|law| law.id.key == BurnClearLawKey::SideRepresentation(TransactionSide::Input))
        .unwrap();
    assert_eq!(input.evaluate(&checked(&common, &extension)), Ok(true));
    if let BurnClearLaw::SideRepresentation { allowed, .. } = &mut input.body {
        allowed.insert(RepresentationMode::PrivateCommitted);
    }
    assert_eq!(
        input.evaluate(&checked(&common, &extension)),
        Err(BurnClearLawError::Identity)
    );
    assert_eq!(
        validate_burn_clear_law_census(OperationId::Burn, &declarations),
        Err(BurnClearLawError::Census)
    );
}

#[test]
fn burn_data_output_disagreement_is_a_binding_refusal() {
    let (common, mut extension) = burn();
    assert!(holds(BurnClearLawKey::DataOutputs, &common, &extension));
    if let BurnClearPayload::Burn { records, .. } = &mut extension.payload {
        records[0].fresh_ash.ordinal = 1;
    }
    assert_eq!(
        validate_burn_clear_observation(&common, &context(&common), Some(&extension)),
        Err(BurnClearRefusal::DataAssociation)
    );
}

#[test]
fn clear_binding_propagates_transition_and_arithmetic_errors() {
    for (fixture, yl, expected) in [
        (
            clear(10, 6, 0, 5, 1, 5),
            0,
            BurnClearRefusal::ClearTransition,
        ),
        (
            clear(10, 6, 3, 6, 0, 4),
            PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1,
            BurnClearRefusal::Arithmetic,
        ),
    ] {
        let (common, mut extension) = fixture;
        assert!(holds(BurnClearLawKey::ClearBinding, &common, &extension));
        if let BurnClearPayload::Clear { predecessor, .. } = &mut extension.payload {
            predecessor.metadata.y_l = amount(yl);
        }
        assert_eq!(
            law(OperationId::Clear, BurnClearLawKey::ClearBinding)
                .evaluate(&checked(&common, &extension)),
            Err(BurnClearLawError::Operand(expected))
        );
    }
}

#[test]
fn data_output_expected_rows_are_checked_before_evaluation() {
    let (common, extension) = burn();
    let mut declaration = law(OperationId::Burn, BurnClearLawKey::DataOutputs);
    assert_eq!(
        declaration.evaluate(&checked(&common, &extension)),
        Ok(true)
    );
    if let BurnClearLaw::DataOutputs { expected } = &mut declaration.body {
        expected.clear();
    }
    assert_eq!(
        declaration.evaluate(&checked(&common, &extension)),
        Err(BurnClearLawError::Identity)
    );
}

#[test]
fn binding_predicates_have_region_and_projection_disagreements() {
    let (mut common, mut extension) = burn();
    assert!(holds(BurnClearLawKey::BurnBinding, &common, &extension));
    common.claim_object(object(ObservedSide::Input, 7, ObjectId::State, 1));
    extension.context = context(&common);
    assert!(!holds(BurnClearLawKey::BurnBinding, &common, &extension));
    let (mut common, mut extension) = clear(10, 6, 3, 6, 0, 4);
    assert!(holds(BurnClearLawKey::ClearBinding, &common, &extension));
    common.projections.insert(ProjectionId::DistributionResidue);
    extension.context = context(&common);
    assert!(!holds(BurnClearLawKey::ClearBinding, &common, &extension));
}

#[test]
fn independent_floor_arithmetic_refusal_has_a_control() {
    let (common, mut extension) = clear(10, 6, 3, 6, 0, 4);
    assert!(holds(BurnClearLawKey::ClearFloor, &common, &extension));
    if let BurnClearPayload::Clear { successor, .. } = &mut extension.payload {
        successor.metadata.y_l = amount(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1);
    }
    let refusal = law(OperationId::Clear, BurnClearLawKey::ClearFloor)
        .evaluate(&checked(&common, &extension))
        .unwrap_err();
    assert!(matches!(refusal, BurnClearLawError::Evaluation(_)));
    assert_eq!(refusal.name(), "evaluation");
}
