use architecture::{ARCHITECTURE, OperationId};

use crate::{RealizationError, RealizationScope, derive};

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
