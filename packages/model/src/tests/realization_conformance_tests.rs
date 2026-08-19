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

    spec.evaluate_operation(observation).unwrap()
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
        realization::RelationKind::AllowedObjectFamilies,
        realization::RelationSubject::TransactionSide {
            side: realization::TransactionSide::Output,
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
    let execution = execute_bound(&world, transition, next_order(&world)).unwrap();
    let observation = observe_compact_ash(&execution).unwrap();
    let observation = observation.observation().clone();
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
    let execution = execute_bound(&world, transition, next_order(&world)).unwrap();
    let observation = observe_compact_ash(&execution).unwrap();
    let observation = observation.observation().clone();

    let mut wrong_amount = observation.clone();
    wrong_amount
        .objects
        .iter_mut()
        .find(|object| object.reference.side == realization::ObservedSide::Output)
        .unwrap()
        .value =
        realization::ObservedValue::Protocol(realization::ProtocolAmount::new(99).unwrap());
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
fn compact_ash_binding_requires_exact_one_transition_extension() {
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
        bind_execution(&world, transition, &not_one).unwrap_err(),
        ExecutionBindingError::InvalidSuccessorExtension,
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
    let execution = execute_bound(&world, transition, next_order(&world)).unwrap();
    let observation = observe_live_transfer(&execution).unwrap();
    let observation = observation.observation().clone();
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
    let split = apply_checked(
        &world,
        &TransferReceipts {
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
        },
        next_order(&world),
    );
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
    let execution = execute_bound(&split, transition, next_order(&split)).unwrap();
    let observation = observe_live_transfer(&execution).unwrap();
    let observation = observation.observation().clone();
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
    let execution = execute_bound(&world, transition, next_order(&world)).unwrap();
    let observation = observe_live_transfer(&execution).unwrap();
    let observation = observation.observation().clone();

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
        .value = realization::ObservedValue::Protocol(realization::ProtocolAmount::new(1).unwrap());
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

fn genesis_live_transfer(world: &World, owner: OwnerKey) -> TransferReceipts {
    let input = find_receipts(world, GENESIS_OWNER, ReceiptClass::Live)[0];

    TransferReceipts {
        class: ReceiptClass::Live,
        inputs: vec![input],
        outputs: vec![ReceiptDestination {
            owner,
            value: world.utxo(input).unwrap().value,
        }],
        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    }
}

#[test]
fn binding_accepts_the_exact_executed_live_transfer() {
    let world = test_fixtures::world();
    let request = genesis_live_transfer(&world, ALICE);
    let after = apply_checked(&world, &request, next_order(&world));
    let bound = bind_execution(&world, request, &after).unwrap();

    assert_eq!(bound.after(), &after);
    assert_eq!(bound.certificate().branch, BranchKind::TransferLive);
    assert!(observe_live_transfer(&bound).is_ok());
}

#[test]
fn binding_rejects_a_successor_from_another_live_transfer() {
    // T1: certificate from the executed request, authorization data from a
    // different same-branch request. Before binding, this hybrid projected
    // and reported conformant.
    let world = test_fixtures::world();
    let executed = genesis_live_transfer(&world, ALICE);
    let unrelated = genesis_live_transfer(&world, BOB);
    assert_ne!(executed, unrelated);

    let after = apply_checked(&world, &executed, next_order(&world));

    assert_eq!(
        bind_execution(&world, unrelated, &after).unwrap_err(),
        ExecutionBindingError::RequestBindingMismatch,
    );
}

#[test]
fn binding_rejects_mutated_live_transfer_requests() {
    let world = test_fixtures::world();
    let executed = genesis_live_transfer(&world, ALICE);
    let after = apply_checked(&world, &executed, next_order(&world));

    let mut wrong_amount = executed.clone();
    wrong_amount.outputs[0].value = sat(1);
    let mut wrong_signers = executed.clone();
    wrong_signers.signers = signers(&[ALICE]);
    let mut wrong_inputs = executed;
    wrong_inputs.inputs.clear();

    for request in [wrong_amount, wrong_signers, wrong_inputs] {
        assert_eq!(
            bind_execution(&world, request, &after).unwrap_err(),
            ExecutionBindingError::RequestBindingMismatch,
        );
    }
}

#[test]
fn binding_rejects_a_successor_from_another_compact_ash() {
    let world = world_with_two_ash();
    let executed = CompactAsh {
        ash_inputs: find_ash(&world),
        fee_envelope: FeeEnvelope::default(),
    };
    let after = apply_checked(&world, &executed, next_order(&world));

    let mut short = executed;
    short.ash_inputs.pop();
    assert_eq!(
        bind_execution(&world, short, &after).unwrap_err(),
        ExecutionBindingError::RequestBindingMismatch,
    );
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

#[test]
fn bound_execution_discharges_the_model_substrate_premise() {
    let world = world_with_two_ash();
    let transition = CompactAsh {
        ash_inputs: find_ash(&world),
        fee_envelope: FeeEnvelope::default(),
    };
    let execution = execute_bound(&world, transition, next_order(&world)).unwrap();
    let observation = observe_compact_ash(&execution).unwrap();
    let report = evaluate(observation.observation());

    // The realization evaluator itself never passes the premise…
    assert!(!report.is_evidence_complete());
    assert!(!report.has_semantic_failure());
    assert_eq!(
        report
            .required_external_evidence()
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            realization::ExternalEvidenceRequirement::SubstrateConservation {
                operation: architecture::OperationId::CompactAsh,
                asset: architecture::AssetId::Lbtc,
            }
        ],
    );

    // …while the bound model execution establishes exactly the
    // model-side copy of it.
    assert!(unresolved_model_evidence(&report, &observation).is_empty());
}

#[test]
fn live_transfer_bound_execution_discharges_the_model_substrate_premise() {
    let world = test_fixtures::world();
    let request = genesis_live_transfer(&world, ALICE);
    let execution = execute_bound(&world, request, next_order(&world)).unwrap();
    let observation = observe_live_transfer(&execution).unwrap();
    let report = evaluate(observation.observation());

    assert!(!report.has_semantic_failure());
    assert!(!report.is_evidence_complete());
    assert!(unresolved_model_evidence(&report, &observation).is_empty());
}

#[test]
fn model_evidence_is_operation_scoped() {
    // Evidence established for one pilot must not discharge the other
    // pilot's requirement.
    let live_world = test_fixtures::world();
    let live_request = genesis_live_transfer(&live_world, ALICE);
    let live_execution = execute_bound(&live_world, live_request, next_order(&live_world)).unwrap();
    let live_observation = observe_live_transfer(&live_execution).unwrap();

    let ash_world = world_with_two_ash();
    let ash_transition = CompactAsh {
        ash_inputs: find_ash(&ash_world),
        fee_envelope: FeeEnvelope::default(),
    };
    let ash_execution = execute_bound(&ash_world, ash_transition, next_order(&ash_world)).unwrap();
    let ash_report = evaluate(observe_compact_ash(&ash_execution).unwrap().observation());

    let unresolved = unresolved_model_evidence(&ash_report, &live_observation);
    assert_eq!(
        unresolved.into_iter().collect::<Vec<_>>(),
        vec![
            realization::ExternalEvidenceRequirement::SubstrateConservation {
                operation: architecture::OperationId::CompactAsh,
                asset: architecture::AssetId::Lbtc,
            }
        ],
    );
}

// --- S2-01: sponsor erasure follows the fee-sponsor flow role. ---
//
// The two Phase-1 pilots use ordinary L-BTC only inside the generic
// fee-sponsor region, so the region-derived projection must agree
// exactly with the family-keyed one it replaces. These tests pin that
// invariance, and pin the fact that the region — not the object family
// — is what decides erasure.

fn sponsored_compact_ash() -> ExecutedTransition<CompactAsh> {
    let world = world_with_two_ash();
    let (world, sponsor_input) = fund_lbtc(&world, SPONSOR, sat(10));
    let transition = CompactAsh {
        ash_inputs: find_ash(&world),
        fee_envelope: fee_envelope_exact(sponsor_input, SPONSOR, sat(10), sat(4)),
    };

    execute_bound(&world, transition, next_order(&world)).unwrap()
}

#[test]
fn sponsored_compact_ash_erases_exactly_the_fee_sponsor_region() {
    let execution = sponsored_compact_ash();
    let observation = observe_compact_ash(&execution).unwrap();
    let observation = observation.observation().clone();

    let sponsor_refs = observation
        .open_flows
        .iter()
        .filter(|flow| flow.kind == architecture::OpenFlowKind::FeeSponsor)
        .flat_map(|flow| flow.sources.iter().chain(&flow.destinations))
        .copied()
        .collect::<std::collections::BTreeSet<_>>();

    assert!(
        !sponsor_refs.is_empty(),
        "the fixture must exercise a real sponsor region"
    );

    for object in &observation.objects {
        let erased = object.value == realization::ObservedValue::SponsorOpaque;

        assert_eq!(
            erased,
            sponsor_refs.contains(&object.reference),
            "{:?} erasure must follow fee-sponsor membership",
            object.reference
        );
    }

    let report = evaluate(&observation);

    assert!(
        report.is_conformant(),
        "{:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

#[test]
fn pilot_ordinary_lbtc_is_sponsor_only() {
    // The invariance argument itself: the pilots never place ordinary
    // L-BTC in a protocol flow, which is why the family-keyed and the
    // role-keyed projections agree on them. When a later operation
    // breaks this coincidence, the role-keyed projection is the one
    // that stays correct.
    let execution = sponsored_compact_ash();
    let observation = observe_compact_ash(&execution).unwrap();

    for object in &observation.observation().objects {
        if object.kind
            == realization::ObservedObjectKind::Declared(architecture::ObjectId::PlainLbtc)
        {
            assert_eq!(object.value, realization::ObservedValue::SponsorOpaque);
        }
    }
}

#[test]
fn a_reference_in_both_regions_is_rejected_before_projection() {
    let mut certificate = TransitionCertificate {
        txid: test_fixtures::txid(77),
        order: CanonicalOrder {
            height: 1,
            tx_index: 0,
        },
        branch: BranchKind::CompactAsh,

        consumed: std::collections::BTreeSet::from([1]),
        created: std::collections::BTreeSet::new(),

        state_edge: None,
        resv_edge: None,
        pace_edge: None,
        entitlement_authority_edge: None,
        distribution_authority_edge: None,

        canonical_partition: CertifiedCanonicalPartition::default(),
        open_flows: vec![
            OpenFlowProjection {
                kind: OpenFlowKind::FeeSponsor,
                source_inputs: vec![1],
                destination_outputs: Vec::new(),
                fee: Sat::ZERO,
            },
            OpenFlowProjection {
                kind: OpenFlowKind::Redemption,
                source_inputs: vec![1],
                destination_outputs: Vec::new(),
                fee: Sat::ZERO,
            },
        ],

        chain_fee: Sat::ZERO,

        burn: None,
        clear: None,
        distribution_residue: None,
    };

    assert_eq!(
        crate::conformance::sponsor_region(&certificate),
        Err(ConformanceProjectionError::SponsorProtocolRegionOverlap(1)),
    );

    // Disjoint regions project: only the fee-sponsor member is erased.
    certificate.open_flows[1].source_inputs = vec![2];
    certificate.consumed.insert(2);

    assert_eq!(
        crate::conformance::sponsor_region(&certificate),
        Ok(std::collections::BTreeSet::from([1])),
    );
}
