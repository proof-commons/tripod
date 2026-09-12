mod compact_ash_tests;
mod constructibility_tests;
mod derivation_tests;
mod disclosure_tests;
mod expression_tests;
mod guide11_reproductions;
mod lifecycle_tests;
mod live_transfer_tests;
mod open_flow_tests;
mod operator_vocabulary_tests;
mod property_graph_tests;
mod relation_identity_tests;
mod root_policy_tests;
mod state_codec_tests;
mod state_tests;
mod type_tests;

use architecture::{ARCHITECTURE, OperationId, semantic_hash, validate_draft};

use crate::{
    ArchitectureBinding, Count, PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE, ProtocolAmount, RealizationError,
    RealizationScope,
};

#[test]
fn architecture_binding_matches_the_typed_architecture() {
    let binding = ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap();

    assert_eq!(
        binding.architecture_schema_version(),
        ARCHITECTURE.document.architecture_schema_version,
    );

    assert_eq!(
        binding.realization_version(),
        ARCHITECTURE.document.realization_version,
    );

    assert_eq!(
        binding.semantic_hash(),
        semantic_hash(&validate_draft(&ARCHITECTURE).unwrap()).unwrap()
    );
}

#[test]
fn protocol_amount_enforces_the_v13_domain() {
    assert_eq!(ProtocolAmount::new(0).unwrap(), ProtocolAmount::ZERO);

    assert_eq!(
        ProtocolAmount::new(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1)
            .unwrap()
            .get(),
        PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1,
    );

    assert_eq!(
        ProtocolAmount::new(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE),
        Err(RealizationError::AmountOutOfDomain {
            value: PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE,
        }),
    );
}

#[test]
fn protocol_amount_arithmetic_reestablishes_the_domain() {
    assert_eq!(
        ProtocolAmount::new(2)
            .unwrap()
            .checked_add(ProtocolAmount::new(3).unwrap())
            .unwrap()
            .get(),
        5,
    );

    let maximum = ProtocolAmount::new(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1).unwrap();

    assert_eq!(
        maximum.checked_add(ProtocolAmount::ONE),
        Err(RealizationError::AmountOutOfDomain {
            value: PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE,
        }),
    );

    assert_eq!(
        ProtocolAmount::ZERO.checked_sub(ProtocolAmount::ONE),
        Err(RealizationError::AmountUnderflow),
    );
}

#[test]
fn counts_are_not_protocol_amounts() {
    let count = Count::new(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE);

    assert_eq!(count.get(), PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE);

    assert_eq!(
        ProtocolAmount::new(count.get()),
        Err(RealizationError::AmountOutOfDomain {
            value: PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE,
        }),
    );
}

#[test]
fn scopes_are_canonical_and_reject_duplicates() {
    let scope =
        RealizationScope::from_operations([OperationId::CompactAsh, OperationId::TransferLive])
            .unwrap();

    assert_eq!(
        scope.operations(),
        &[OperationId::TransferLive, OperationId::CompactAsh],
    );

    assert_eq!(
        RealizationScope::from_operations([OperationId::CompactAsh, OperationId::CompactAsh]),
        Err(RealizationError::DuplicateScopeOperation(
            OperationId::CompactAsh,
        )),
    );
}

#[test]
fn phase1_scope_is_explicitly_partial() {
    let scope = RealizationScope::phase1_pilots();

    assert!(scope.contains(OperationId::CompactAsh));
    assert!(scope.contains(OperationId::TransferLive));
    assert!(!scope.contains(OperationId::Burn));

    let error = scope.try_complete(&ARCHITECTURE).unwrap_err();

    let RealizationError::IncompleteScope { missing } = error else {
        panic!("Phase-1 scope must fail specifically as incomplete");
    };

    assert_ne!(missing, [] as [architecture::OperationId; 0]);
    assert!(missing.contains(&OperationId::Burn));
    assert!(missing.contains(&OperationId::Cycle));
}

#[test]
fn complete_architecture_scope_upgrades() {
    let scope =
        RealizationScope::from_operations(architecture::OperationId::ALL.iter().copied()).unwrap();

    let complete = scope.try_complete(&ARCHITECTURE).unwrap();

    assert_eq!(complete.as_scope().operations(), scope.operations());
}
