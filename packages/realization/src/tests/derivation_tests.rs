use std::collections::BTreeMap;

use architecture::{ARCHITECTURE, OperationId};

use crate::{
    ArchitectureBinding, ConstructibilityNode, ConstructibilityNodeId, DisclosureNodeId,
    DisclosureReason, DisclosureSeed, ExprId, ExpressionDeclaration, ExpressionNode,
    ExpressionRole, FactId, LifecycleNodeId, OperationRealization, RealizationError,
    RealizationScope, RelationDependencyDeclaration, RelationEdge, SemanticType, derive,
    derive::assemble_scoped_realization, project_scoped_realization,
};

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
fn undeclared_burn_scope_rejects_for_now() {
    let error = derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::Burn]).unwrap(),
    )
    .unwrap_err();

    assert_eq!(
        error,
        RealizationError::UnsupportedOperationDeclaration(OperationId::Burn),
    );
}
