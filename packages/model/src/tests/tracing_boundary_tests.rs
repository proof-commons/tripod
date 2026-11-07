//! Scoped tracing assertions for model execution boundaries.

use super::scenario_fixtures::{find_receipts, next_order, signers};
use super::test_fixtures;
use super::tracing_capture::{capture, events, no_warn_or_error};
use crate::*;

fn announcement(maturity_cycle: u64) -> AnnounceMaturity {
    AnnounceMaturity {
        maturity_cycle,
        signers: signers(&[OPERATOR_KEY]),
        fee_envelope: FeeEnvelope::default(),
    }
}

#[test]
fn genesis_control_and_one_coordinate_refusal() {
    if !super::tracing_capture::isolated_capture_child() {
        return;
    }
    let constants = test_fixtures::constants();
    let order = CanonicalOrder {
        height: 0,
        tx_index: 0,
    };
    let txid = test_fixtures::txid(0);
    let plain = genesis(
        constants.clone(),
        test_fixtures::sat(1_000_000),
        order,
        txid,
    );
    let (traced, records) = capture(|| {
        genesis(
            constants.clone(),
            test_fixtures::sat(1_000_000),
            order,
            txid,
        )
    });
    assert_eq!(traced, plain);
    assert_eq!(events(&records, "model genesis completed").len(), 1);
    assert_eq!(
        events(&records, "model genesis completed")[0]["fields"]["outcome"],
        "completed"
    );
    let (refused, records) = capture(|| genesis(constants, Sat::ZERO, order, txid));
    assert_eq!(refused, Err(Guard::Domain));
    assert_eq!(
        refused,
        genesis(test_fixtures::constants(), Sat::ZERO, order, txid)
    );
    assert_eq!(
        events(&records, "model genesis completed")[0]["fields"]["refusal_kind"],
        "Domain"
    );
    assert!(no_warn_or_error(&records));
}

#[test]
fn transition_and_binding_control_and_refusal() {
    if !super::tracing_capture::isolated_capture_child() {
        return;
    }
    let world = test_fixtures::world();
    let cycle = world.constants.min_maturity_lead;
    let request = announcement(cycle);
    let order = test_fixtures::order(1);
    let plain = execute(&world, &request, order);
    let (traced, records) = capture(|| execute(&world, &request, order));
    assert_eq!(traced, plain);
    assert!(traced.is_ok());
    assert_eq!(
        events(&records, "model transition completed")[0]["fields"]["outcome"],
        "completed"
    );
    let short = announcement(cycle - 1);
    let (refused, records) = capture(|| execute(&world, &short, order));
    assert_eq!(refused, Err(Guard::MaturityLeadTooShort));
    assert_eq!(refused, execute(&world, &short, order));
    assert_eq!(
        events(&records, "model transition completed")[0]["fields"]["refusal_kind"],
        "MaturityLeadTooShort"
    );
    assert!(no_warn_or_error(&records));

    let after = plain.unwrap();
    let plain = bind_execution(&world, request.clone(), &after);
    let (traced, records) = capture(|| bind_execution(&world, request.clone(), &after));
    assert_eq!(traced, plain);
    assert_eq!(
        events(&records, "model binding completed")[0]["fields"]["outcome"],
        "completed"
    );
    let mut altered = after;
    altered.history.transitions.pop();
    let (refused, records) = capture(|| bind_execution(&world, request.clone(), &altered));
    assert_eq!(
        refused,
        Err(ExecutionBindingError::InvalidSuccessorExtension)
    );
    assert_eq!(refused, bind_execution(&world, request, &altered));
    assert_eq!(
        events(&records, "model binding completed")[0]["fields"]["refusal_kind"],
        "invalid_successor_extension"
    );
    assert!(no_warn_or_error(&records));
}

#[test]
fn invariant_control_and_single_fault_diagnostic() {
    if !super::tracing_capture::isolated_capture_child() {
        return;
    }
    let world = test_fixtures::world();
    let plain = check_invariant(&world);
    let (traced, records) = capture(|| check_invariant(&world));
    assert_eq!(traced, plain);
    assert_eq!(
        events(&records, "model invariant checked")[0]["fields"]["outcome"],
        "completed"
    );
    let mut corrupted = world;
    corrupted.roots.state = u64::MAX;
    let plain = check_invariant(&corrupted);
    let (traced, records) = capture(|| check_invariant(&corrupted));
    assert_eq!(traced, plain);
    assert!(traced.is_err());
    assert_eq!(events(&records, "model invariant failed").len(), 0);
    assert!(no_warn_or_error(&records));
    assert_eq!(
        events(&records, "model invariant checked")[0]["fields"]["outcome"],
        "failed"
    );
    assert_eq!(
        events(&records, "model invariant checked")[0]["level"],
        "DEBUG"
    );
    assert_eq!(
        events(&records, "model invariant checked")[0]["fields"]["error_kind"],
        plain.unwrap_err().name(),
    );
}

#[test]
fn execute_invariant_fault_emits_one_diagnostic() {
    if !super::tracing_capture::isolated_capture_child() {
        return;
    }
    let mut world = test_fixtures::world();
    let request = announcement(world.constants.min_maturity_lead);
    world.history.genesis.state_out = u64::MAX;
    let order = test_fixtures::order(1);
    let applied = request.apply(&world, order).unwrap();
    let invariant_error = check_invariant(&applied).unwrap_err();
    let plain = execute(&world, &request, order);
    let (traced, records) = capture(|| execute(&world, &request, order));
    assert_eq!(traced, plain);
    assert_eq!(traced, Err(Guard::InvariantFailure));
    assert_eq!(events(&records, "model invariant failed").len(), 1);
    assert_eq!(
        events(&records, "model invariant failed")[0]["fields"]["error_kind"],
        invariant_error.name(),
    );
    assert_eq!(events(&records, "model transition completed").len(), 1);
    assert_eq!(
        events(&records, "model transition completed")[0]["fields"]["outcome"],
        "failed",
    );
    assert_eq!(
        events(&records, "model transition completed")[0]["level"],
        "DEBUG"
    );
}

#[test]
fn live_projection_control_and_class_refusal() {
    if !super::tracing_capture::isolated_capture_child() {
        return;
    }
    let world = test_fixtures::world();
    let request_for = |class| {
        let input = find_receipts(&world, GENESIS_OWNER, class)[0];
        TransferReceipts {
            class,
            inputs: vec![input],
            outputs: vec![ReceiptDestination {
                owner: GENESIS_OWNER,
                value: world.utxo(input).unwrap().value,
            }],
            signers: signers(&[GENESIS_OWNER]),
            fee_envelope: FeeEnvelope::default(),
        }
    };
    let live = execute_bound(&world, request_for(ReceiptClass::Live), next_order(&world)).unwrap();
    let plain = observe_live_transfer(&live);
    let (traced, records) = capture(|| observe_live_transfer(&live));
    assert_eq!(traced, plain);
    assert!(traced.is_ok());
    assert_eq!(
        events(&records, "model projection completed")[0]["fields"]["outcome"],
        "completed"
    );
    let locked = execute_bound(
        &world,
        request_for(ReceiptClass::TimeLocked),
        next_order(&world),
    )
    .unwrap();
    let plain = observe_live_transfer(&locked);
    let (traced, records) = capture(|| observe_live_transfer(&locked));
    assert_eq!(traced, plain);
    assert!(matches!(
        traced,
        Err(ConformanceProjectionError::WrongBranch { .. })
    ));
    assert_eq!(
        events(&records, "model projection completed")[0]["fields"]["refusal_kind"],
        "wrong_branch"
    );
    assert!(no_warn_or_error(&records));
}

struct DoneScheduler;

impl MaintenanceScheduler for DoneScheduler {
    fn next_action(&self, _world: &World) -> Result<Option<MaintenanceAction>, Guard> {
        Ok(None)
    }

    fn fee_envelope_for(
        &self,
        _world: &World,
        _action: &MaintenanceAction,
    ) -> Result<FeeEnvelope, Guard> {
        Ok(FeeEnvelope::default())
    }
}

#[test]
fn maintenance_drivers_record_terminal_summaries() {
    if !super::tracing_capture::isolated_capture_child() {
        return;
    }
    let world = test_fixtures::world();
    let plain = drive_shared_state_to_fixpoint(&world, &DoneScheduler, || next_order(&world));
    let (traced, records) =
        capture(|| drive_shared_state_to_fixpoint(&world, &DoneScheduler, || next_order(&world)));
    assert_eq!(traced, plain);
    assert_eq!(events(&records, "model maintenance finished").len(), 1);
    assert_eq!(
        events(&records, "model maintenance finished")[0]["fields"]["step_count"],
        0
    );

    let plain = drive_quiescence_with_scheduler(&world, &DoneScheduler, 1, false);
    let (traced, records) =
        capture(|| drive_quiescence_with_scheduler(&world, &DoneScheduler, 1, false));
    assert_eq!(traced, plain);
    assert_eq!(events(&records, "model maintenance finished").len(), 1);

    let plain = drive_sponsored_quiescence(&world, 100);
    let (traced, records) = capture(|| drive_sponsored_quiescence(&world, 100));
    assert_eq!(traced, plain);
    assert_eq!(events(&records, "model maintenance finished").len(), 1);

    let plain = drive_shared_state_sweepability(&world, 100);
    let (traced, records) = capture(|| drive_shared_state_sweepability(&world, 100));
    assert_eq!(traced, plain);
    assert_eq!(events(&records, "model maintenance finished").len(), 1);
}

#[test]
fn property_trace_counts_applied_and_rejected_actions() {
    if !super::tracing_capture::isolated_capture_child() {
        return;
    }
    let world = test_fixtures::world();
    let actions = || {
        vec![
            PropertyAction::AdvanceBlocks { blocks: 1 },
            PropertyAction::AdvanceBlocks { blocks: u64::MAX },
        ]
    };
    let plain = drive_property_trace(&world, actions());
    let (traced, records) = capture(|| drive_property_trace(&world, actions()));
    assert_eq!(traced, plain);
    let event = events(&records, "model property trace completed")[0];
    assert_eq!(event["fields"]["applied_count"], 1);
    assert_eq!(event["fields"]["rejected_count"], 1);
    assert_eq!(event["fields"]["step_count"], 2);
    assert!(no_warn_or_error(&records));

    let plain = drive_property_seed_trace(&world, std::iter::empty());
    let (traced, records) = capture(|| drive_property_seed_trace(&world, std::iter::empty()));
    assert_eq!(traced, plain);
    assert_eq!(
        events(&records, "model property trace completed")[0]["fields"]["step_count"],
        0
    );
}
