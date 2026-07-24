use architecture::{ARCHITECTURE, OperationId};

use crate::{
    ArchitectureBinding, OperationRealization, RealizationError, RealizationScope, derive,
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
