//! Model-to-realization conformance tests.
//!
//! These tests execute the model independently, project primitive observations,
//! and then evaluate realization relations. Model transition acceptance does not
//! delegate to realization.

use super::advanced_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

fn phase1_realization() -> realization::ScopedRealizationSpec {
    realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::phase1_pilots(),
    )
    .unwrap()
}

fn evaluate(observation: &realization::OperationObservation) -> realization::ConformanceReport {
    let spec = phase1_realization();

    realization::evaluate_operation(
        &spec.relation_graph,
        &spec.relation_node_by_id,
        &spec.relation_evaluation_order,
        observation,
    )
    .unwrap()
}

fn failed(report: &realization::ConformanceReport, relation: &realization::RelationId) -> bool {
    report.verdicts.iter().any(|verdict| {
        verdict.relation == *relation
            && matches!(
                verdict.status,
                realization::RelationStatus::Failed { .. }
                    | realization::RelationStatus::Blocked { .. }
            )
    })
}

fn relation_id(
    operation: architecture::OperationId,
    kind: realization::RelationKind,
    subject: realization::RelationSubject,
) -> realization::RelationId {
    realization::RelationId::new(operation, kind, subject)
}

fn compact_conservation() -> realization::RelationId {
    relation_id(
        architecture::OperationId::CompactAsh,
        realization::RelationKind::Conservation,
        realization::RelationSubject::Asset {
            asset: architecture::AssetId::U,
        },
    )
}

fn compact_projection_policy() -> realization::RelationId {
    relation_id(
        architecture::OperationId::CompactAsh,
        realization::RelationKind::ProjectionPolicy,
        realization::RelationSubject::Operation,
    )
}

fn live_authorization() -> realization::RelationId {
    relation_id(
        architecture::OperationId::TransferLive,
        realization::RelationKind::Authorization,
        realization::RelationSubject::ObjectFamily {
            side: realization::TransactionSide::Input,
            object: architecture::ObjectId::ReceiptLive,
        },
    )
}

fn live_output_closure() -> realization::RelationId {
    relation_id(
        architecture::OperationId::TransferLive,
        realization::RelationKind::OutputClosure,
        realization::RelationSubject::ObjectFamily {
            side: realization::TransactionSide::Output,
            object: architecture::ObjectId::ReceiptLive,
        },
    )
}

fn live_conservation() -> realization::RelationId {
    relation_id(
        architecture::OperationId::TransferLive,
        realization::RelationKind::Conservation,
        realization::RelationSubject::Asset {
            asset: architecture::AssetId::U,
        },
    )
}

fn live_representation() -> realization::RelationId {
    relation_id(
        architecture::OperationId::TransferLive,
        realization::RelationKind::Representation,
        realization::RelationSubject::Representation {
            object: architecture::ObjectId::ReceiptLive,
        },
    )
}

fn world_with_two_ash() -> World {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));
    let first = burn_from_owner(&world, ALICE, sat(50), Vec::new());
    let with_bob = give_live_receipt(&first, BOB, sat(50));

    burn_from_owner(&with_bob, BOB, sat(50), Vec::new())
}

#[test]
fn compact_ash_model_transition_satisfies_realization() {
    let world = world_with_two_ash();
    let transition = CompactAsh {
        ash_inputs: find_ash(&world),
        fee_envelope: FeeEnvelope::default(),
    };
    let after = transition.apply(&world, next_order(&world)).unwrap();
    let observation = observe_compact_ash(&world, &transition, &after).unwrap();
    let report = evaluate(&observation);

    assert!(
        report.is_conformant(),
        "{:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

#[test]
fn compact_ash_observation_mutations_are_load_bearing() {
    let world = world_with_two_ash();
    let transition = CompactAsh {
        ash_inputs: find_ash(&world),
        fee_envelope: FeeEnvelope::default(),
    };
    let after = transition.apply(&world, next_order(&world)).unwrap();
    let observation = observe_compact_ash(&world, &transition, &after).unwrap();

    let mut wrong_amount = observation.clone();
    wrong_amount
        .objects
        .iter_mut()
        .find(|object| object.reference.side == realization::ObservedSide::Output)
        .unwrap()
        .value = realization::ProtocolAmount::new(99).unwrap();
    assert!(failed(&evaluate(&wrong_amount), &compact_conservation()));

    let mut burn_projection = observation;
    burn_projection
        .projections
        .insert(architecture::ProjectionId::BurnEvent);
    assert!(failed(
        &evaluate(&burn_projection),
        &compact_projection_policy()
    ));
}

#[test]
fn compact_ash_observation_requires_exact_one_transition_extension() {
    let world = world_with_two_ash();
    let transition = CompactAsh {
        ash_inputs: find_ash(&world),
        fee_envelope: FeeEnvelope::default(),
    };
    let after = transition.apply(&world, next_order(&world)).unwrap();
    let mut not_one = after.clone();
    not_one
        .history
        .transitions
        .push(after.history.transitions.last().unwrap().clone());

    assert_eq!(
        observe_compact_ash(&world, &transition, &not_one),
        Err(ConformanceProjectionError::NotOneTransitionExtension),
    );
}

#[test]
fn live_transfer_split_satisfies_realization() {
    let world = test_fixtures::world();
    let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];
    let value = world.utxo(input).unwrap().value;
    let first = sat(value.get() / 2);
    let second = value.checked_sub(first).unwrap();
    let transition = TransferReceipts {
        class: ReceiptClass::Live,
        inputs: vec![input],
        outputs: vec![
            ReceiptDestination {
                owner: ALICE,
                value: first,
            },
            ReceiptDestination {
                owner: BOB,
                value: second,
            },
        ],
        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    };
    let after = transition.apply(&world, next_order(&world)).unwrap();
    let observation = observe_live_transfer(&world, &transition, &after).unwrap();
    let report = evaluate(&observation);

    assert!(
        report.is_conformant(),
        "{:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

#[test]
fn live_transfer_merge_satisfies_realization() {
    let world = test_fixtures::world();
    let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];
    let split = TransferReceipts {
        class: ReceiptClass::Live,
        inputs: vec![input],
        outputs: vec![
            ReceiptDestination {
                owner: ALICE,
                value: sat(5),
            },
            ReceiptDestination {
                owner: BOB,
                value: sat(7),
            },
            ReceiptDestination {
                owner: GENESIS_OWNER,
                value: world
                    .utxo(input)
                    .unwrap()
                    .value
                    .checked_sub(sat(12))
                    .unwrap(),
            },
        ],
        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();
    let alice = find_receipts(&split, ALICE, ReceiptClass::Live)[0];
    let bob = find_receipts(&split, BOB, ReceiptClass::Live)[0];
    let transition = TransferReceipts {
        class: ReceiptClass::Live,
        inputs: vec![alice, bob],
        outputs: vec![ReceiptDestination {
            owner: CAROL,
            value: sat(12),
        }],
        signers: signers(&[ALICE, BOB]),
        fee_envelope: FeeEnvelope::default(),
    };
    let after = transition.apply(&split, next_order(&split)).unwrap();
    let observation = observe_live_transfer(&split, &transition, &after).unwrap();
    let report = evaluate(&observation);

    assert!(
        report.is_conformant(),
        "{:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

#[test]
fn live_transfer_observation_mutations_are_load_bearing() {
    let world = test_fixtures::world();
    let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];
    let transition = TransferReceipts {
        class: ReceiptClass::Live,
        inputs: vec![input],
        outputs: vec![ReceiptDestination {
            owner: ALICE,
            value: world.utxo(input).unwrap().value,
        }],
        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    };
    let after = transition.apply(&world, next_order(&world)).unwrap();
    let observation = observe_live_transfer(&world, &transition, &after).unwrap();

    let mut missing_signer = observation.clone();
    missing_signer.protocol_signers.clear();
    assert!(failed(&evaluate(&missing_signer), &live_authorization()));

    let mut wrong_output = observation.clone();
    wrong_output
        .objects
        .iter_mut()
        .find(|object| object.reference.side == realization::ObservedSide::Output)
        .unwrap()
        .kind = realization::ObservedObjectKind::Declared(architecture::ObjectId::Ash);
    assert!(failed(&evaluate(&wrong_output), &live_output_closure()));

    let mut wrong_amount = observation.clone();
    wrong_amount
        .objects
        .iter_mut()
        .find(|object| object.reference.side == realization::ObservedSide::Output)
        .unwrap()
        .value = realization::ProtocolAmount::new(1).unwrap();
    assert!(failed(&evaluate(&wrong_amount), &live_conservation()));

    let mut wrong_representation = observation;
    wrong_representation
        .objects
        .iter_mut()
        .find(|object| object.reference.side == realization::ObservedSide::Output)
        .unwrap()
        .representation = realization::RepresentationMode::PublicCommitted;
    assert!(failed(
        &evaluate(&wrong_representation),
        &live_representation()
    ));
}

#[test]
fn phase1_declassification_matches_model_owned_pilot_rows() {
    let analysis = realization::phase1_declassification();

    assert!(analysis.newly_disclosed.is_empty());
    assert!(!analysis.required_public.is_empty());
    assert!(analysis.retained_private.iter().any(|fact| {
        matches!(
            fact,
            realization::FactId::FamilyAmount {
                operation: architecture::OperationId::TransferLive,
                object: architecture::ObjectId::ReceiptLive,
                ..
            }
        )
    }));

    for operation in ["compact-ash", "transfer-live-receipts"] {
        let row = crate::artifacts::declassification_rows()
            .into_iter()
            .find(|row| row.operation == operation)
            .expect("pilot row exists");

        assert_eq!(row.declassifies, [] as [std::string::String; 0]);
    }
}
