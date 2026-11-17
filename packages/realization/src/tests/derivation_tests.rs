use std::collections::{BTreeMap, BTreeSet};

use architecture::{ARCHITECTURE, BoundId, ObjectId, OperationId};

use crate::{
    ArchitectureBinding, ConstructibilityNode, ConstructibilityNodeId,
    DisclosureDependencyProjection, DisclosureNodeId, DisclosureReason, DisclosureSeed, ExprId,
    ExpressionDeclaration, ExpressionNode, ExpressionRole, FactId, InitialVisibility,
    LifecycleDependencyProjection, LifecycleEdge, LifecycleNodeId, OperationRealization,
    RealizationError, RealizationScope, RelationDependencyDeclaration, RelationEdge,
    RepresentationMode, SemanticType, derive, derive::assemble_scoped_realization,
    project_scoped_realization,
};

// Literal lifecycle census of the adopted ASH, live receipt and STATE tables.
pub(super) fn assert_literal_lifecycle(
    realization: &crate::ScopedRealizationSpec,
    objects: &[ObjectId],
) {
    use ObjectId::{Ash, ReceiptLive, State};
    use OperationId::{
        AdmitDeposits, AnnounceMaturity, Burn, Clear, CompactAsh, Cycle, ReceiptRelabel, Redeem,
        TransferLive,
    };
    use RepresentationMode::{Explicit, PrivateCommitted, PublicCommitted};
    let mut nodes = BTreeSet::new();
    let mut edges = BTreeSet::new();
    let table: [(ObjectId, &[RepresentationMode], &[OperationId]); 3] = [
        (Ash, &[Explicit, PublicCommitted], &[CompactAsh, Clear]),
        (
            ReceiptLive,
            &[Explicit, PrivateCommitted],
            &[TransferLive, Burn, Redeem],
        ),
        (
            State,
            &[Explicit, PublicCommitted],
            &[
                AdmitDeposits,
                Cycle,
                Redeem,
                ReceiptRelabel,
                Clear,
                AnnounceMaturity,
            ],
        ),
    ];
    for (object, modes, exits) in table {
        if !objects.contains(&object) {
            continue;
        }
        for mode in modes {
            let source = LifecycleNodeId::Representation {
                object,
                mode: *mode,
            };
            nodes.insert(source.clone());
            for exit in exits {
                let target = LifecycleNodeId::RequiredExit {
                    object,
                    operation: *exit,
                };
                nodes.insert(target.clone());
                edges.insert(LifecycleDependencyProjection {
                    source: source.clone(),
                    target,
                    edge: LifecycleEdge::RequiresExit,
                });
            }
        }
    }
    let projection = project_scoped_realization(realization);
    assert_eq!(
        projection
            .lifecycle
            .nodes
            .into_iter()
            .map(|node| node.id)
            .collect::<BTreeSet<_>>(),
        nodes
    );
    assert_eq!(
        projection
            .lifecycle
            .edges
            .into_iter()
            .collect::<BTreeSet<_>>(),
        edges
    );
}

/// Compare the assembled pair with the pre-union single-declaration paths.
pub(super) fn assert_pair_disclosure_union(
    mixed: &crate::ScopedRealizationSpec,
    pair: [OperationId; 2],
) {
    let mut expected_nodes = BTreeMap::new();
    let mut expected_edges = BTreeSet::new();
    let mut expected_seeds = Vec::new();
    let mut expected_analysis = crate::DeclassificationAnalysis::default();
    for operation in pair {
        let singleton = derive(
            &ARCHITECTURE,
            RealizationScope::from_operations([operation]).unwrap(),
        )
        .unwrap();
        let declaration = &singleton.operations[&operation];
        assert_eq!(mixed.operations.get(&operation), Some(declaration));
        let (graph, index) = crate::declassification::build_disclosure_graph(
            declaration.disclosure_nodes.clone(),
            declaration.disclosure_edges.clone(),
        )
        .unwrap();
        for node in graph.node_weights() {
            if let Some(previous) = expected_nodes.insert(node.id(), (*node).clone()) {
                assert_eq!(&previous, node, "shared fact visibility changed");
            }
        }
        expected_edges.extend(crate::declassification::project_disclosure_graph(&graph).edges);
        expected_seeds.extend(declaration.disclosure_seeds.iter().cloned());
        let analysis = crate::declassification::analyze_disclosure(
            &graph,
            &index,
            &declaration.disclosure_seeds,
        )
        .unwrap();
        for (fact, reasons) in analysis.required_public {
            expected_analysis
                .required_public
                .entry(fact)
                .or_default()
                .extend(reasons);
        }
        for (fact, reasons) in analysis.newly_disclosed {
            expected_analysis
                .newly_disclosed
                .entry(fact)
                .or_default()
                .extend(reasons);
        }
        expected_analysis
            .retained_private
            .extend(analysis.retained_private);
    }
    let projection = project_scoped_realization(mixed);
    assert_eq!(
        projection.disclosure.nodes,
        expected_nodes.keys().cloned().collect::<Vec<_>>()
    );
    assert_eq!(
        projection.disclosure.edges,
        expected_edges.into_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        mixed
            .disclosure_graph
            .node_weights()
            .cloned()
            .map(|node| (node.id(), node))
            .collect::<BTreeMap<_, _>>(),
        expected_nodes,
    );
    expected_seeds.sort();
    let mut actual_seeds = mixed
        .operations
        .values()
        .flat_map(|declaration| declaration.disclosure_seeds.iter().cloned())
        .collect::<Vec<_>>();
    actual_seeds.sort();
    assert_eq!(actual_seeds, expected_seeds);
    assert_eq!(projection.declassification, expected_analysis);
}

#[test]
#[allow(clippy::too_many_lines)] // Literal graph, disclosure, and permutation expectations form one combined-scope control.
fn combined_declared_scope_has_literal_union() {
    use OperationId::{AnnounceMaturity, Burn, Clear, CompactAsh, TransferLive};
    let ids = [CompactAsh, TransferLive, AnnounceMaturity, Burn, Clear];
    let scope = RealizationScope::from_operations(ids).unwrap();
    let baseline = derive(&ARCHITECTURE, scope.clone()).unwrap();
    baseline.validate_against(&ARCHITECTURE).unwrap();
    assert_literal_lifecycle(
        &baseline,
        &[ObjectId::Ash, ObjectId::ReceiptLive, ObjectId::State],
    );
    let projection = project_scoped_realization(&baseline);
    assert_eq!(projection.operations.len(), 5);
    assert_eq!(
        projection
            .disclosure
            .nodes
            .iter()
            .filter_map(|id| {
                if let DisclosureNodeId::Fact(FactId::BoundValue { bound }) = id {
                    Some(*bound)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            BoundId::AshBatchMax,
            BoundId::BurnInputMax,
            BoundId::BurnChangeMax,
            BoundId::BurnRecordMax,
            BoundId::FeeSponsorInputMax,
        ])
    );
    let declared_nodes = baseline
        .operations
        .values()
        .flat_map(|operation| {
            operation
                .disclosure_nodes
                .iter()
                .map(crate::DisclosureNode::id)
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        projection
            .disclosure
            .nodes
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        declared_nodes
    );
    let declared_edges = baseline
        .operations
        .values()
        .flat_map(|operation| operation.disclosure_edges.iter())
        .map(|row| DisclosureDependencyProjection {
            source: row.source.clone(),
            target: row.target.clone(),
            edge: row.edge,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        projection
            .disclosure
            .edges
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        declared_edges
    );
    let clear_reason = DisclosureReason::PermissionlessConstructibility {
        operation: Clear,
        relation: crate::RelationId::new(
            Clear,
            crate::RelationKind::Constructibility,
            crate::RelationSubject::Operation,
        ),
    };
    let compact_reason = DisclosureReason::PermissionlessConstructibility {
        operation: CompactAsh,
        relation: crate::RelationId::new(
            CompactAsh,
            crate::RelationKind::Constructibility,
            crate::RelationSubject::Operation,
        ),
    };
    assert_eq!(
        projection.declassification.required_public[&FactId::BoundValue {
            bound: BoundId::AshBatchMax
        }],
        BTreeSet::from([clear_reason.clone(), compact_reason])
    );
    assert_eq!(
        projection.declassification.required_public[&FactId::BoundValue {
            bound: BoundId::FeeSponsorInputMax
        }],
        BTreeSet::from([clear_reason])
    );
    assert_eq!(
        projection.declassification.retained_private,
        BTreeSet::from([
            FactId::FamilyAmount {
                operation: Burn,
                side: crate::TransactionSide::Output,
                object: ObjectId::ReceiptLive
            },
            FactId::FamilyAmount {
                operation: TransferLive,
                side: crate::TransactionSide::Input,
                object: ObjectId::ReceiptLive
            },
            FactId::FamilyAmount {
                operation: TransferLive,
                side: crate::TransactionSide::Output,
                object: ObjectId::ReceiptLive
            },
        ])
    );
    assert!(projection.declassification.newly_disclosed.is_empty());
    // Scope construction sorts operation IDs. Reversal checks that public
    // scope-input order canonicalizes; it cannot vary assembler iteration.
    let reversed = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations(ids.into_iter().rev()).unwrap(),
    )
    .unwrap();
    assert_eq!(projection, project_scoped_realization(&reversed));
    let mut operations = baseline.operations;
    for operation in operations.values_mut() {
        operation.lifecycle_nodes.reverse();
        operation.lifecycle_edges.reverse();
        operation.disclosure_nodes.reverse();
    }
    let permuted = assemble_scoped_realization(
        &ARCHITECTURE,
        ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        scope,
        operations,
    )
    .unwrap();
    assert_eq!(projection, project_scoped_realization(&permuted));
}

#[test]
fn standalone_operation_projections_are_unchanged_by_union() {
    use OperationId::{AnnounceMaturity, Burn, Clear, CompactAsh, TransferLive};
    for (operation, objects) in [
        (CompactAsh, vec![ObjectId::Ash]),
        (TransferLive, vec![ObjectId::ReceiptLive]),
        (AnnounceMaturity, vec![ObjectId::State]),
        (Burn, vec![ObjectId::Ash, ObjectId::ReceiptLive]),
        (Clear, vec![ObjectId::Ash, ObjectId::State]),
    ] {
        let realization = derive(
            &ARCHITECTURE,
            RealizationScope::from_operations([operation]).unwrap(),
        )
        .unwrap();
        assert_literal_lifecycle(&realization, &objects);
        let row = &realization.operations[&operation];
        // These unchanged single-list builders are the pre-union path for one
        // declaration; compare the complete affected projections and analysis.
        let (lifecycle, _, _) = crate::lifecycle::build_lifecycle_graph(
            row.lifecycle_nodes.clone(),
            row.lifecycle_edges.clone(),
        )
        .unwrap();
        let (disclosure, disclosure_nodes) = crate::declassification::build_disclosure_graph(
            row.disclosure_nodes.clone(),
            row.disclosure_edges.clone(),
        )
        .unwrap();
        let expected_analysis = crate::declassification::analyze_disclosure(
            &disclosure,
            &disclosure_nodes,
            &row.disclosure_seeds,
        )
        .unwrap();
        let projection = project_scoped_realization(&realization);
        assert_eq!(
            projection.lifecycle,
            crate::lifecycle::project_lifecycle_graph(&lifecycle)
        );
        assert_eq!(
            projection.disclosure,
            crate::declassification::project_disclosure_graph(&disclosure)
        );
        assert_eq!(projection.declassification, expected_analysis);
    }
    let burn = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([Burn]).unwrap(),
    )
    .unwrap();
    assert_eq!(
        burn.declassification.retained_private,
        BTreeSet::from([FactId::FamilyAmount {
            operation: Burn,
            side: crate::TransactionSide::Output,
            object: ObjectId::ReceiptLive,
        }])
    );
    let clear = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([Clear]).unwrap(),
    )
    .unwrap();
    assert!(clear.declassification.retained_private.is_empty());
    assert_eq!(
        clear
            .disclosure_graph
            .node_weights()
            .filter(|node| matches!(
                node,
                crate::DisclosureNode::Fact {
                    id: FactId::BoundValue {
                        bound: BoundId::AshBatchMax
                    },
                    initial_visibility: InitialVisibility::Public
                }
            ))
            .count(),
        1
    );
}

#[test]
fn compact_ash_scope_derives_direct_graphs() {
    let realization = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::CompactAsh]).unwrap(),
    )
    .unwrap();

    assert!(
        realization
            .operations
            .contains_key(&OperationId::CompactAsh)
    );
    assert_eq!(realization.expression_graph.node_count(), 0);
    assert!(!realization.relation_node_by_id.is_empty());
    assert_eq!(realization.scope.operations(), &[OperationId::CompactAsh]);
}

#[test]
fn phase1_scope_derives_both_pilots() {
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();

    assert!(
        realization
            .operations
            .contains_key(&OperationId::CompactAsh)
    );
    assert!(
        realization
            .operations
            .contains_key(&OperationId::TransferLive)
    );
}

#[test]
fn repeated_phase1_derivation_is_equal() {
    let first = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();
    let second = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();

    assert_eq!(
        project_scoped_realization(&first),
        project_scoped_realization(&second),
    );
}

#[test]
fn operation_scope_order_does_not_change_the_result() {
    let first = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .unwrap(),
    )
    .unwrap();
    let second = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::TransferLive, OperationId::CompactAsh])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(
        project_scoped_realization(&first),
        project_scoped_realization(&second),
    );
}

#[test]
fn declaration_order_permutations_preserve_the_complete_projection() {
    // F3-005: the stable projection must not depend on the source
    // order of any set-like declaration collection. One family is
    // reversed at a time so a regression names the family, and the
    // permuted declarations are rebuilt through the shared assembly
    // and validation path.
    type FamilyReversal = (&'static str, fn(&mut OperationRealization));

    let baseline = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();
    let reversals: [FamilyReversal; 10] = [
        ("expressions", |operation| operation.expressions.reverse()),
        ("relations", |operation| operation.relations.reverse()),
        ("relation_dependencies", |operation| {
            operation.relation_dependencies.reverse();
        }),
        ("constructibility_nodes", |operation| {
            operation.constructibility_nodes.reverse();
        }),
        ("constructibility_edges", |operation| {
            operation.constructibility_edges.reverse();
        }),
        ("lifecycle_nodes", |operation| {
            operation.lifecycle_nodes.reverse();
        }),
        ("lifecycle_edges", |operation| {
            operation.lifecycle_edges.reverse();
        }),
        ("disclosure_nodes", |operation| {
            operation.disclosure_nodes.reverse();
        }),
        ("disclosure_edges", |operation| {
            operation.disclosure_edges.reverse();
        }),
        ("disclosure_seeds", |operation| {
            operation.disclosure_seeds.reverse();
        }),
    ];

    for (family, reverse) in reversals {
        let mut operations = baseline.operations.clone();
        for declaration in operations.values_mut() {
            reverse(declaration);
        }
        let permuted = assemble_scoped_realization(
            &ARCHITECTURE,
            ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
            RealizationScope::phase1_pilots(),
            operations,
        )
        .unwrap_or_else(|error| panic!("{family}: rebuild failed: {error:?}"));

        assert_eq!(
            project_scoped_realization(&baseline),
            project_scoped_realization(&permuted),
            "reversing {family} moved the stable projection",
        );
    }
}

// --- F3-006 generic ownership validation: a mistyped declaration
// must fail with a focused typed ownership error before graph
// assembly, so no semantic invariant rests on helper-constructor
// correctness. Each mutation rebuilds through the shared assembly
// path. ---

fn phase1_operations() -> BTreeMap<OperationId, OperationRealization> {
    derive(&ARCHITECTURE, RealizationScope::phase1_pilots())
        .unwrap()
        .operations
}

fn assemble_phase1(
    operations: BTreeMap<OperationId, OperationRealization>,
) -> Result<crate::ScopedRealizationSpec, RealizationError> {
    assemble_scoped_realization(
        &ARCHITECTURE,
        ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        RealizationScope::phase1_pilots(),
        operations,
    )
}

#[test]
fn declaration_returned_for_the_wrong_operation_is_rejected() {
    let mut operations = phase1_operations();
    let compact = operations.remove(&OperationId::CompactAsh).unwrap();
    // The compact-ASH declaration comes back under the live-transfer
    // key, as if derive_operation returned the wrong declaration.
    let live = operations.insert(OperationId::TransferLive, compact);
    assert!(live.is_some());

    let error = assemble_scoped_realization(
        &ARCHITECTURE,
        ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        RealizationScope::from_operations([OperationId::TransferLive]).unwrap(),
        operations,
    )
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::OperationDeclarationIdentityMismatch {
            requested: OperationId::TransferLive,
            declared: OperationId::CompactAsh,
        },
    );
}

#[test]
fn foreign_relation_in_an_operation_declaration_is_rejected() {
    let mut operations = phase1_operations();
    let foreign = operations[&OperationId::TransferLive].relations[0].clone();
    let foreign_id = foreign.id.clone();
    operations
        .get_mut(&OperationId::CompactAsh)
        .unwrap()
        .relations
        .push(foreign);

    let error = assemble_phase1(operations).unwrap_err();
    assert_eq!(
        error,
        RealizationError::ForeignRelationOwnership {
            operation: OperationId::CompactAsh,
            relation: foreign_id,
        },
    );
}

#[test]
fn proof_alternative_bound_to_another_relation_is_rejected() {
    let mut operations = phase1_operations();
    let compact = operations.get_mut(&OperationId::CompactAsh).unwrap();
    let foreign_owner = compact.relations[1].id.clone();
    let carrying = &mut compact.relations[0];
    let carrying_id = carrying.id.clone();
    // The alternative names a different relation of the same
    // operation: binding, not just operation ownership, is checked.
    let stray = carrying.proof_alternatives.iter().next().unwrap().proof();
    carrying
        .proof_alternatives
        .insert(crate::ProofAlternativeId::new(foreign_owner.clone(), stray));

    let error = assemble_phase1(operations).unwrap_err();
    assert_eq!(
        error,
        RealizationError::ForeignProofAlternativeBinding {
            relation: carrying_id,
            foreign: foreign_owner,
        },
    );
}

#[test]
fn foreign_expression_identity_is_rejected() {
    let mut operations = phase1_operations();
    // Neither pilot declares standing expressions, so the foreign
    // expression is constructed: its identity is owned by a
    // live-transfer relation while compact ASH declares it.
    let foreign_relation = operations[&OperationId::TransferLive].relations[0]
        .id
        .clone();
    let foreign_id = ExprId::relation(foreign_relation, ExpressionRole::Condition);
    operations
        .get_mut(&OperationId::CompactAsh)
        .unwrap()
        .expressions
        .push(ExpressionDeclaration {
            id: foreign_id.clone(),
            ty: SemanticType::Bool,
            node: ExpressionNode::Bool(true),
        });

    let error = assemble_phase1(operations).unwrap_err();
    assert_eq!(
        error,
        RealizationError::ForeignExpressionOwnership {
            operation: OperationId::CompactAsh,
            expression: foreign_id,
        },
    );
}

#[test]
fn expression_reading_another_operations_fact_is_rejected() {
    let mut operations = phase1_operations();
    let compact = operations.get_mut(&OperationId::CompactAsh).unwrap();
    let owned_relation = compact.relations[0].id.clone();
    let foreign_fact = FactId::Signers {
        operation: OperationId::TransferLive,
    };
    // The expression identity is owned by compact ASH; only its fact
    // payload reaches across operations.
    compact.expressions.push(ExpressionDeclaration {
        id: ExprId::relation(owned_relation, ExpressionRole::Condition),
        ty: SemanticType::OwnerSet,
        node: ExpressionNode::Fact(foreign_fact.clone()),
    });

    let error = assemble_phase1(operations).unwrap_err();
    assert_eq!(
        error,
        RealizationError::ForeignExpressionOwnership {
            operation: OperationId::CompactAsh,
            expression: ExprId::fact(foreign_fact),
        },
    );
}

#[test]
fn relation_dependency_on_a_foreign_relation_is_rejected() {
    let mut operations = phase1_operations();
    let foreign = operations[&OperationId::TransferLive].relations[0]
        .id
        .clone();
    let compact = operations.get_mut(&OperationId::CompactAsh).unwrap();
    let owned = compact.relations[0].id.clone();
    compact
        .relation_dependencies
        .push(RelationDependencyDeclaration {
            prerequisite: foreign.clone(),
            dependent: owned,
            edge: RelationEdge::RecognitionBeforeCardinality,
        });

    let error = assemble_phase1(operations).unwrap_err();
    assert_eq!(
        error,
        RealizationError::ForeignRelationDependency {
            operation: OperationId::CompactAsh,
            relation: foreign,
        },
    );
}

#[test]
fn foreign_constructibility_node_is_rejected() {
    let mut operations = phase1_operations();
    let node = ConstructibilityNodeId::Operation(OperationId::TransferLive);
    operations
        .get_mut(&OperationId::CompactAsh)
        .unwrap()
        .constructibility_nodes
        .push(ConstructibilityNode { id: node.clone() });

    let error = assemble_phase1(operations).unwrap_err();
    assert_eq!(
        error,
        RealizationError::ForeignConstructibilityOwnership {
            operation: OperationId::CompactAsh,
            node,
        },
    );
}

#[test]
fn disclosure_seed_naming_a_foreign_relation_is_rejected() {
    let mut operations = phase1_operations();
    let foreign = operations[&OperationId::TransferLive].relations[0]
        .id
        .clone();
    operations
        .get_mut(&OperationId::CompactAsh)
        .unwrap()
        .disclosure_seeds
        .push(DisclosureSeed {
            node: DisclosureNodeId::Relation(foreign.clone()),
            reason: DisclosureReason::PublicInterface,
        });

    let error = assemble_phase1(operations).unwrap_err();
    assert_eq!(
        error,
        RealizationError::ForeignDisclosureOwnership {
            operation: OperationId::CompactAsh,
            node: DisclosureNodeId::Relation(foreign),
        },
    );
}

#[test]
fn cross_operation_lifecycle_exits_remain_valid_typed_payloads() {
    // A RequiredExit names its exit operation as semantic content.
    // Compact ASH declares a Clear exit and live transfer declares
    // Burn and Redeem exits; ownership validation must keep deriving
    // them, because the declaring relation stays operation-owned.
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();

    let mut compact_exits = realization.operations[&OperationId::CompactAsh]
        .lifecycle_nodes
        .iter()
        .filter_map(|node| match &node.id {
            LifecycleNodeId::RequiredExit { operation, .. } => Some(*operation),
            LifecycleNodeId::Representation { .. } => None,
        });
    assert!(compact_exits.any(|operation| operation == OperationId::Clear));
}

#[test]
fn burn_is_supported_while_redeem_remains_undeclared() {
    let control = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::Burn]).unwrap(),
    )
    .unwrap();
    assert!(control.operations.contains_key(&OperationId::Burn));
    let error = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::Redeem]).unwrap(),
    )
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::UnsupportedOperationDeclaration(OperationId::Redeem),
    );
}

#[test]
fn clear_is_supported_while_redeem_remains_undeclared() {
    let control = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::Clear]).unwrap(),
    )
    .unwrap();
    assert!(control.operations.contains_key(&OperationId::Clear));
    assert_eq!(
        derive(
            &ARCHITECTURE,
            RealizationScope::from_operations([OperationId::Redeem]).unwrap(),
        )
        .unwrap_err(),
        RealizationError::UnsupportedOperationDeclaration(OperationId::Redeem),
    );
}

// --- S2 disclosure/relation weld: ownership validation proves only
// that a named relation WOULD belong to the declaring operation. These
// mutations name a relation that carries the right operation but was
// never declared, so ownership passes and only the census weld can
// reject them. ---

/// A relation key owned by compact ASH that the pilot never declares.
fn phantom_relation() -> crate::RelationId {
    crate::RelationId::new(
        OperationId::CompactAsh,
        crate::RelationKind::Conservation,
        crate::RelationSubject::ObjectFamily {
            side: crate::TransactionSide::Input,
            object: architecture::ObjectId::PlainLbtc,
        },
    )
}

#[test]
fn phantom_disclosure_relation_node_is_rejected() {
    let mut operations = phase1_operations();
    let relation = phantom_relation();
    assert!(
        !operations[&OperationId::CompactAsh]
            .relations
            .iter()
            .any(|declared| declared.id == relation),
        "the fixture relation must genuinely be undeclared"
    );

    operations
        .get_mut(&OperationId::CompactAsh)
        .unwrap()
        .disclosure_nodes
        .push(crate::DisclosureNode::Relation {
            id: relation.clone(),
        });

    let error = assemble_phase1(operations).unwrap_err();
    assert_eq!(
        error,
        RealizationError::UnknownDisclosureRelation { relation },
    );
}

#[test]
fn phantom_relation_in_a_disclosure_seed_reason_is_rejected() {
    // The seed's node may be perfectly ordinary; the phantom hides in
    // the reason, which the graph never mentions.
    let mut operations = phase1_operations();
    let relation = phantom_relation();
    let declaration = operations.get_mut(&OperationId::CompactAsh).unwrap();
    let node = declaration.disclosure_nodes[0].id();

    declaration.disclosure_seeds.push(crate::DisclosureSeed {
        node,
        reason: crate::DisclosureReason::TargetSafety {
            relation: relation.clone(),
        },
    });

    let error = assemble_phase1(operations).unwrap_err();
    assert_eq!(
        error,
        RealizationError::UnknownDisclosureRelation { relation },
    );
}

#[test]
fn declared_disclosure_relations_still_derive() {
    // The weld must not reject the pilots' own relation-bearing
    // disclosure declarations.
    assemble_phase1(phase1_operations()).expect("phase-1 pilots derive unchanged");
}

// --- S5: evaluation order is a valid schedule, not a canonical fact.
// The spec keeps one because the evaluator needs it; the projection
// must not, because among independent nodes the chosen order is an
// artifact of graph traversal rather than a property of the
// relations. ---

#[test]
fn evaluation_order_is_absent_from_the_stable_projection() {
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();
    let projection = project_scoped_realization(&realization);

    // The spec still schedules evaluation; the projection does not
    // publish that schedule. Asserted through the debug rendering
    // because absence of a field cannot be named directly.
    assert!(
        !format!("{projection:?}").contains("evaluation_order"),
        "the stable projection must not publish a traversal order",
    );
    assert!(
        !realization.relation_evaluation_order.is_empty(),
        "the spec must still carry a schedule for the evaluator",
    );
}

#[test]
fn a_different_valid_schedule_leaves_the_projection_equal() {
    // Reversing declaration order permutes graph insertion, which is
    // exactly what can hand the toposort a different — equally valid —
    // schedule among independent nodes. The projection must not move,
    // and the relation and expression graphs must still be equal as
    // graphs.
    let baseline = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();

    let mut operations = baseline.operations.clone();
    for declaration in operations.values_mut() {
        declaration.relations.reverse();
        declaration.expressions.reverse();
    }
    let permuted = assemble_phase1(operations).expect("permuted rebuild");

    assert_eq!(
        project_scoped_realization(&baseline),
        project_scoped_realization(&permuted),
        "a different valid schedule moved the stable projection",
    );
}

#[test]
fn announcement_singleton_derivation_is_repeatable() {
    let first = super::announce_maturity_tests::realization();
    let second = super::announce_maturity_tests::realization();
    assert_eq!(first.scope.operations(), &[OperationId::AnnounceMaturity]);
    assert_eq!(first.project(), second.project());
}

#[test]
fn three_operation_scope_is_order_independent() {
    let operations = [
        OperationId::AnnounceMaturity,
        OperationId::CompactAsh,
        OperationId::TransferLive,
    ];
    let first = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations(operations).unwrap(),
    )
    .unwrap();
    let second = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations(operations.into_iter().rev()).unwrap(),
    )
    .unwrap();
    assert_eq!(first.project(), second.project());
}

#[test]
fn three_operation_declarations_are_permutation_stable() {
    let realization = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([
            OperationId::AnnounceMaturity,
            OperationId::CompactAsh,
            OperationId::TransferLive,
        ])
        .unwrap(),
    )
    .unwrap();
    super::announce_maturity_tests::assert_permutations(&realization);
}
