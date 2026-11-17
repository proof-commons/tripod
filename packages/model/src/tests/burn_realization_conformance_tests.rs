//! Independent accepted Burn executions against the complete realization declaration.

use super::advanced_fixtures::give_live_receipt;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;
use realization::{BurnClearPayload, BurnClearRefusal, ObservedSide, RecordCredit, RelationStatus};

fn burn_request(
    world: &World,
    owners: &[OwnerKey],
    change: &[u64],
    records: &[(u32, AttestationAddress, u64)],
    fee_envelope: FeeEnvelope,
) -> BurnReceipts {
    let mut used = std::collections::BTreeMap::<OwnerKey, usize>::new();
    let receipts = owners
        .iter()
        .map(|owner| {
            let index = used.entry(*owner).or_default();
            let receipt = find_receipts(world, *owner, ReceiptClass::Live)[*index];
            *index += 1;
            receipt
        })
        .collect::<Vec<_>>();
    let input = receipts
        .iter()
        .map(|outpoint| world.utxo(*outpoint).unwrap().value.get())
        .sum::<u64>();
    let change_total = change.iter().sum::<u64>();
    BurnReceipts {
        receipts,
        signers: signers(owners),
        ash_value: sat(input - change_total),
        change: change
            .iter()
            .enumerate()
            .map(|(index, amount)| ReceiptDestination {
                owner: owners[index % owners.len()],
                value: sat(*amount),
            })
            .collect(),
        records: records
            .iter()
            .map(|(record_index, address, amount)| BurnRecord {
                record_index: *record_index,
                address: *address,
                amount: sat(*amount),
            })
            .collect(),
        fee_envelope,
    }
}

fn bound_projection(
    world: &World,
    request: BurnReceipts,
) -> (ModelBurnConformanceObservation, bool) {
    let execution = execute_bound(world, request, next_order(world)).unwrap();
    let projected = observe_burn(&execution).unwrap();
    assert_eq!(
        execution
            .after()
            .history
            .transitions
            .last()
            .unwrap()
            .burn
            .as_ref()
            .unwrap()
            .records,
        execution.request().records
    );
    let indexer = ReferenceIndexer::from_assumed_kernel_history(
        &execution.after().history,
        &test_fixtures::chain_view_for_history(execution.after()),
        [0; 32],
    )
    .unwrap();
    let model_burn = indexer
        .burns()
        .get(&execution.certificate().txid)
        .expect("indexed bound burn");
    assert_eq!(
        model_burn.records,
        execution.certificate().burn.as_ref().unwrap().records
    );
    (projected, model_burn.records_accepted().unwrap())
}

fn bind(projected: &ModelBurnConformanceObservation) -> realization::BoundBurnClearObservation {
    realization::validate_burn_clear_observation(
        projected.common.observation(),
        &projected.extension.context,
        Some(&projected.extension),
    )
    .unwrap()
}

fn assert_agrees(
    projected: &ModelBurnConformanceObservation,
    model_credited: bool,
    expected: RecordCredit,
) {
    let bound = bind(projected);
    realization::validate_burn_event(&bound, &projected.event).unwrap();
    realization::validate_accounting(&bound, &projected.accounting).unwrap();
    let actual = realization::evaluate_record_credit(&bound).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(
        matches!(actual, RecordCredit::Credited { .. }),
        model_credited
    );
    let spec = realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::from_operations([architecture::OperationId::Burn]).unwrap(),
    )
    .unwrap();
    let report = spec.evaluate_operation_with_burn_clear(&bound).unwrap();
    assert!(
        report.is_conformant(),
        "{:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
    assert!(report.verdicts.iter().any(|verdict| matches!(
        verdict.status,
        RelationStatus::EvidenceRequired {
            requirement: realization::ExternalEvidenceRequirement::SubstrateConservation { .. }
        }
    )));
    assert!(unresolved_model_evidence(&report, &projected.common).is_empty());
    assert!(!report.is_evidence_complete());
}

#[test]
fn accepted_burn_matrix_agrees_with_complete_declaration() {
    // Each row is a model execution first; realization sees only its projection.
    let mut alice = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));
    let (one, model_credited) = bound_projection(
        &alice,
        burn_request(&alice, &[ALICE], &[], &[], FeeEnvelope::default()),
    );
    assert_agrees(&one, model_credited, RecordCredit::Credited { total: 0 });

    let (credited, model_credited) = bound_projection(
        &alice,
        burn_request(
            &alice,
            &[ALICE],
            &[20],
            &[(0, ADDRESS_A, 50)],
            FeeEnvelope::default(),
        ),
    );
    assert_agrees(
        &credited,
        model_credited,
        RecordCredit::Credited { total: 50 },
    );

    let (exact, model_credited) = bound_projection(
        &alice,
        burn_request(
            &alice,
            &[ALICE],
            &[20],
            &[(0, ADDRESS_A, 30), (1, ADDRESS_B, 50)],
            FeeEnvelope::default(),
        ),
    );
    assert_agrees(&exact, model_credited, RecordCredit::Credited { total: 80 });

    alice = give_live_receipt(&alice, BOB, sat(60));
    let (multiple, model_credited) = bound_projection(
        &alice,
        burn_request(
            &alice,
            &[ALICE, BOB],
            &[10, 10],
            &[(0, ADDRESS_A, 30), (1, ADDRESS_B, 40)],
            FeeEnvelope::default(),
        ),
    );
    assert_agrees(
        &multiple,
        model_credited,
        RecordCredit::Credited { total: 70 },
    );

    let repeated_world = give_live_receipt(&alice, ALICE, sat(40));
    let (repeated, model_credited) = bound_projection(
        &repeated_world,
        burn_request(
            &repeated_world,
            &[ALICE, ALICE],
            &[5, 5],
            &[(0, ADDRESS_A, 70), (1, ADDRESS_A, 70)],
            FeeEnvelope::default(),
        ),
    );
    assert_agrees(
        &repeated,
        model_credited,
        RecordCredit::Overclaim {
            total: 140,
            fresh_amount: realization::ProtocolAmount::new(130).unwrap(),
        },
    );

    let large = realization::PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1;
    let (beyond, model_credited) = bound_projection(
        &alice,
        burn_request(
            &alice,
            &[ALICE],
            &[],
            &[(0, ADDRESS_A, large), (1, ADDRESS_A, large)],
            FeeEnvelope::default(),
        ),
    );
    assert_agrees(
        &beyond,
        model_credited,
        RecordCredit::Overclaim {
            total: 2 * u128::from(large),
            fresh_amount: realization::ProtocolAmount::new(100).unwrap(),
        },
    );

    let (sponsored_world, sponsor_input) = fund_lbtc(&alice, SPONSOR, sat(20));
    let (sponsored, model_credited) = bound_projection(
        &sponsored_world,
        burn_request(
            &sponsored_world,
            &[ALICE],
            &[10],
            &[(0, ADDRESS_B, 10)],
            fee_envelope_exact(sponsor_input, SPONSOR, sat(20), sat(1)),
        ),
    );
    assert_agrees(
        &sponsored,
        model_credited,
        RecordCredit::Credited { total: 10 },
    );
    assert!(
        sponsored
            .common
            .observation()
            .objects
            .iter()
            .any(|object| object.value == realization::ObservedValue::SponsorOpaque)
    );
}

#[test]
fn binding_and_projection_refusals_keep_their_boundaries() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));
    let request = burn_request(
        &world,
        &[ALICE],
        &[20],
        &[(0, ADDRESS_A, 50)],
        FeeEnvelope::default(),
    );
    let executed = execute_bound(&world, request.clone(), next_order(&world)).unwrap();
    let mut wrong = executed.after().clone();
    wrong
        .history
        .transitions
        .push(wrong.history.transitions.last().unwrap().clone());
    assert_eq!(
        bind_execution(&world, request, &wrong).unwrap_err(),
        ExecutionBindingError::InvalidSuccessorExtension
    );

    let projected = observe_burn(&executed).unwrap();
    let bound = bind(&projected);
    let mut changed_source = projected.common.observation().clone();
    changed_source
        .objects
        .iter_mut()
        .find(|object| {
            object.reference.side == ObservedSide::Input
                && object.kind
                    == realization::ObservedObjectKind::Declared(
                        architecture::ObjectId::ReceiptLive,
                    )
        })
        .unwrap()
        .value = realization::ObservedValue::Protocol(realization::ProtocolAmount::new(1).unwrap());
    assert_eq!(
        realization::validate_burn_clear_observation(
            &changed_source,
            &projected.extension.context,
            Some(&projected.extension)
        )
        .unwrap_err(),
        BurnClearRefusal::CommonObservation
    );

    let mut changed_record = projected.extension.clone();
    let BurnClearPayload::Burn { records, .. } = &mut changed_record.payload else {
        unreachable!()
    };
    records[0].fresh_ash = realization::ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 0,
    };
    assert_eq!(
        realization::validate_burn_clear_observation(
            projected.common.observation(),
            &projected.extension.context,
            Some(&changed_record)
        )
        .unwrap_err(),
        BurnClearRefusal::DataAssociation
    );

    let mut wrong_ordinal = projected.extension.clone();
    let BurnClearPayload::Burn { records, .. } = &mut wrong_ordinal.payload else {
        unreachable!()
    };
    records[0].ordinal = 1;
    assert_eq!(
        realization::validate_burn_clear_observation(
            projected.common.observation(),
            &projected.extension.context,
            Some(&wrong_ordinal)
        )
        .unwrap_err(),
        BurnClearRefusal::RecordOrdinal
    );

    let mut wrong_event = projected.event.clone();
    wrong_event.records[0].amount = realization::ProtocolAmount::new(51).unwrap();
    assert_eq!(
        realization::validate_burn_event(&bound, &wrong_event),
        Err(BurnClearRefusal::BurnEvent)
    );
    let mut wrong_accounting = projected.accounting.clone();
    wrong_accounting.values = realization::AccountingValues::Burn {
        record_total: 51,
        fresh_amount: projected.event.fresh_amount,
    };
    assert_eq!(
        realization::validate_accounting(&bound, &wrong_accounting),
        Err(BurnClearRefusal::Accounting)
    );
}

#[test]
fn equal_total_changed_split_passes_standalone_and_disagrees_with_model() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));
    let (projected, model_credited) = bound_projection(
        &world,
        burn_request(&world, &[ALICE], &[10, 10], &[], FeeEnvelope::default()),
    );
    assert_agrees(
        &projected,
        model_credited,
        RecordCredit::Credited { total: 0 },
    );
    let mut changed = projected.common.observation().clone();
    let mut changes = changed
        .objects
        .iter_mut()
        .filter(|object| {
            object.reference.side == ObservedSide::Output
                && object.kind
                    == realization::ObservedObjectKind::Declared(
                        architecture::ObjectId::ReceiptLive,
                    )
        })
        .collect::<Vec<_>>();
    assert_eq!(changes.len(), 2);
    changes[0].value =
        realization::ObservedValue::Protocol(realization::ProtocolAmount::new(9).unwrap());
    changes[1].value =
        realization::ObservedValue::Protocol(realization::ProtocolAmount::new(11).unwrap());
    let mut offered = projected.extension.clone();
    offered.context =
        realization::BurnClearContext::bind_observation(changed.clone(), [0; 32], [1; 32]).unwrap();
    let changed_bound =
        realization::validate_burn_clear_observation(&changed, &offered.context, Some(&offered))
            .unwrap();
    let spec = realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::from_operations([architecture::OperationId::Burn]).unwrap(),
    )
    .unwrap();
    assert!(
        spec.evaluate_operation_with_burn_clear(&changed_bound)
            .unwrap()
            .is_conformant()
    );
    assert_eq!(
        realization::validate_burn_clear_observation(
            projected.common.observation(),
            &projected.extension.context,
            Some(&offered)
        ),
        Err(BurnClearRefusal::CommonObservation)
    );
    assert_eq!(
        realization::validate_burn_event(&changed_bound, &projected.event),
        Err(BurnClearRefusal::BurnEvent)
    );
}

#[test]
fn burn_entry_requires_carrier_scope_and_model_bounds() {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));
    let (projected, _) = bound_projection(
        &world,
        burn_request(&world, &[ALICE], &[], &[], FeeEnvelope::default()),
    );
    let bound = bind(&projected);
    let burn_spec = realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::from_operations([architecture::OperationId::Burn]).unwrap(),
    )
    .unwrap();
    assert_eq!(
        burn_spec.evaluate_operation(projected.common.observation()),
        Err(realization::RealizationError::BurnClearObservation(
            BurnClearRefusal::MissingExtension
        ))
    );
    let pilot_spec = realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::phase1_pilots(),
    )
    .unwrap();
    assert_eq!(
        pilot_spec.evaluate_operation_with_burn_clear(&bound),
        Err(realization::RealizationError::OperationOutsideScope(
            architecture::OperationId::Burn
        ))
    );
    for bound_id in [
        architecture::BoundId::BurnInputMax,
        architecture::BoundId::BurnChangeMax,
        architecture::BoundId::FeeSponsorInputMax,
    ] {
        let mut missing = projected.common.observation().clone();
        missing.bounds.remove(&bound_id);
        let mut offered = projected.extension.clone();
        offered.context =
            realization::BurnClearContext::bind_observation(missing.clone(), [0; 32], [1; 32])
                .unwrap();
        let missing_bound = realization::validate_burn_clear_observation(
            &missing,
            &offered.context,
            Some(&offered),
        )
        .unwrap();
        assert_eq!(
            burn_spec.evaluate_operation_with_burn_clear(&missing_bound),
            Err(realization::RealizationError::MissingBoundValue(bound_id))
        );
    }
}
