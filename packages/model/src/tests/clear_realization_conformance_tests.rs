//! Accepted Clear executions projected independently into the complete declaration.

use super::advanced_fixtures::{burn_from_owner, sealing_world};
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;
use architecture::{BoundId, OperationId};
use realization::{
    AccountingValues, BurnClearPayload, BurnClearRefusal, RelationKind, RelationStatus,
    RelationSubject, StateField,
};

fn execute(world: &World, fee_envelope: FeeEnvelope) -> ModelClearConformanceObservation {
    let execution = execute_bound(
        world,
        ClearAsh {
            ash_inputs: find_ash(world),
            fee_envelope,
        },
        next_order(world),
    )
    .unwrap();
    observe_clear(&execution).unwrap()
}

fn partial_world() -> World {
    let (world, _) = sealing_world();
    let supply = world.state().unwrap().1.y().unwrap();
    burn_from_owner(&world, GENESIS_OWNER, supply, Vec::new())
}

fn bind(projected: &ModelClearConformanceObservation) -> realization::BoundBurnClearObservation {
    realization::validate_burn_clear_observation(
        projected.common.observation(),
        &projected.extension.context,
        Some(&projected.extension),
    )
    .unwrap()
}

fn spec() -> realization::ScopedRealizationSpec {
    realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::from_operations([OperationId::Clear]).unwrap(),
    )
    .unwrap()
}

fn report(projected: &ModelClearConformanceObservation) -> realization::ConformanceReport {
    spec()
        .evaluate_operation_with_burn_clear(&bind(projected))
        .unwrap()
}

fn assert_agrees(projected: &ModelClearConformanceObservation) {
    let bound = bind(projected);
    realization::validate_clear_event(&bound, &projected.event).unwrap();
    realization::validate_accounting(&bound, &projected.accounting).unwrap();
    realization::validate_state_succession(&bound, &projected.state_succession).unwrap();
    let result = report(projected);
    assert!(
        result.is_conformant(),
        "{:?}",
        result.failed_relations().collect::<Vec<_>>()
    );
    assert!(unresolved_model_evidence(&result, &projected.common).is_empty());
    assert!(!result.is_evidence_complete());
}

fn context_for(
    execution: &ExecutedTransition<ClearAsh>,
    common: realization::OperationObservation,
) -> realization::BurnClearContext {
    let (input, output) = match execution.certificate().state_edge.unwrap() {
        RootEdge::Succ { input, output } => (input, output),
        RootEdge::Term { .. } => unreachable!(),
    };
    let transaction = execution.certificate().txid.0;
    let mut certificate = [0; 32];
    certificate[..8].copy_from_slice(&input.to_le_bytes());
    certificate[8..16].copy_from_slice(&output.to_le_bytes());
    certificate[16..].copy_from_slice(&transaction[..16]);
    realization::BurnClearContext::bind_observation(common, transaction, certificate).unwrap()
}

fn rebound(
    projected: &ModelClearConformanceObservation,
    changed: realization::OperationObservation,
    execution: &ExecutedTransition<ClearAsh>,
) -> realization::BoundBurnClearObservation {
    assert_eq!(
        context_for(execution, projected.common.observation().clone()),
        projected.extension.context
    );
    let mut extension = projected.extension.clone();
    extension.context = context_for(execution, changed.clone());
    realization::validate_burn_clear_observation(&changed, &extension.context, Some(&extension))
        .unwrap()
}

fn assert_matrix_case(
    projected: &ModelClearConformanceObservation,
    aggregate: u64,
    live: u64,
    floor_capacity: u64,
    clear_amount: u64,
    residual: u64,
) {
    let predecessor = projected.state_succession.predecessor.metadata;
    assert_eq!(projected.event.aggregate.get(), aggregate);
    assert_eq!(predecessor.y_l.get(), live);
    assert_eq!(
        predecessor.y_l.get() + predecessor.y_t.get() - 1,
        floor_capacity
    );
    assert_eq!(projected.event.clear_amount.get(), clear_amount);
    assert_eq!(
        projected
            .event
            .residual
            .iter()
            .map(|(_, value)| value.get())
            .sum::<u64>(),
        residual
    );
    assert_agrees(projected);
}

fn assert_reversed_request_case() {
    let world = super::ash_clear_tests::world_with_two_ash();
    let mut ash_inputs = find_ash(&world);
    ash_inputs.reverse();
    assert!(ash_inputs[0] > ash_inputs[1]);
    let execution = execute_bound(
        &world,
        ClearAsh {
            ash_inputs,
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    )
    .unwrap();
    let reversed = observe_clear(&execution).unwrap();
    assert_matrix_case(&reversed, 100, 500_000, 999_999, 100, 0);
    assert_eq!(
        reversed.event.ash_sources,
        [
            realization::ObservedObjectRef {
                side: realization::ObservedSide::Input,
                ordinal: 1
            },
            realization::ObservedObjectRef {
                side: realization::ObservedSide::Input,
                ordinal: 2
            },
        ]
    );
}

#[test]
fn clear_model_matrix_conforms() {
    // B strictly limits a full clear; a strictly limiting y_l is unreachable
    // in an accepted history and remains in the branch-local function probe.
    let full = execute(
        &super::ash_clear_tests::world_with_two_ash(),
        FeeEnvelope::default(),
    );
    assert_matrix_case(&full, 100, 500_000, 999_999, 100, 0);
    assert_eq!(full.event.residual, []);

    // Y - 1 strictly limits this partial clear; B and y_l tie above it.
    let partial = execute(&partial_world(), FeeEnvelope::default());
    assert_matrix_case(&partial, 1_000_000, 1_000_000, 999_999, 999_999, 1);
    assert_eq!(partial.event.residual.len(), 1);

    // B = y_l below the floor capacity.
    let world = test_fixtures::world();
    let live = world.state().unwrap().1.y_l;
    let live_tie = execute(
        &burn_from_owner(&world, GENESIS_OWNER, live, Vec::new()),
        FeeEnvelope::default(),
    );
    assert_matrix_case(&live_tie, 500_000, 500_000, 999_999, 500_000, 0);

    // B = Y - 1 below y_l.
    let (world, _) = sealing_world();
    let floor = world
        .state()
        .unwrap()
        .1
        .y()
        .unwrap()
        .checked_sub(Sat::ONE)
        .unwrap();
    let floor_tie = execute(
        &burn_from_owner(&world, GENESIS_OWNER, floor, Vec::new()),
        FeeEnvelope::default(),
    );
    assert_matrix_case(&floor_tie, 999_999, 1_000_000, 999_999, 999_999, 0);

    // B = y_l = Y - 1.
    let tiny = genesis(
        test_fixtures::constants(),
        sat(2),
        CanonicalOrder {
            height: 0,
            tx_index: 0,
        },
        test_fixtures::txid(0),
    )
    .unwrap();
    let three_way = execute(
        &burn_from_owner(&tiny, GENESIS_OWNER, Sat::ONE, Vec::new()),
        FeeEnvelope::default(),
    );
    assert_matrix_case(&three_way, 1, 1, 1, 1, 0);

    // Three ASH retain the strictly B-limited class.
    let world = super::ash_clear_tests::world_with_three_ash();
    let three = execute(&world, FeeEnvelope::default());
    assert_matrix_case(&three, 130, 500_000, 999_999, 130, 0);

    // The sponsor stays opaque and does not change Clear's quantities.
    let (sponsored, sponsor_input) = fund_lbtc(&world, SPONSOR, sat(20));
    let projected = execute(
        &sponsored,
        fee_envelope_exact(sponsor_input, SPONSOR, sat(20), sat(1)),
    );
    assert_matrix_case(&projected, 130, 500_000, 999_999, 130, 0);
    assert!(
        projected
            .common
            .observation()
            .objects
            .iter()
            .any(|object| object.value == realization::ObservedValue::SponsorOpaque)
    );

    // An accepted descending request must still bind ASH in certificate order.
    assert_reversed_request_case();
}

#[test]
fn clear_successor_six_fields_match_both_branches() {
    for world in [
        super::ash_clear_tests::world_with_two_ash(),
        partial_world(),
    ] {
        let before = world.state().unwrap().1;
        let execution = execute_bound(
            &world,
            ClearAsh {
                ash_inputs: find_ash(&world),
                fee_envelope: FeeEnvelope::default(),
            },
            next_order(&world),
        )
        .unwrap();
        let after = execution.after().state().unwrap().1;
        let projected = observe_clear(&execution).unwrap();
        let pair = &projected.state_succession;
        assert_eq!(
            pair.predecessor.metadata,
            super::state_metadata_conformance_tests::project(before)
        );
        assert_eq!(
            pair.successor.metadata,
            super::state_metadata_conformance_tests::project(after)
        );
        realization::validate_state_succession(&bind(&projected), pair).unwrap();
        assert_agrees(&projected);
    }
}

#[test]
fn clear_floor_uses_actual_successor_supply() {
    let projected = execute(&partial_world(), FeeEnvelope::default());
    assert_agrees(&projected);
    let floor = realization::burn_clear_laws::burn_clear_laws(OperationId::Clear)
        .into_iter()
        .find(|law| law.id.key == realization::burn_clear_laws::BurnClearLawKey::ClearFloor)
        .unwrap();
    assert_eq!(floor.evaluate(&bind(&projected)), Ok(true));
    assert_eq!(
        projected.state_succession.successor.metadata.y_l.get()
            + projected.state_succession.successor.metadata.y_t.get(),
        1
    );
    assert!(projected.event.clear_amount.get() > 0);
    let mut changed = projected.extension.clone();
    if let BurnClearPayload::Clear { successor, .. } = &mut changed.payload {
        successor.metadata.y_l = realization::ProtocolAmount::ZERO;
    }
    let bound = realization::validate_burn_clear_observation(
        projected.common.observation(),
        &projected.extension.context,
        Some(&changed),
    )
    .unwrap();
    assert_eq!(floor.evaluate(&bound), Ok(false));
    let report = spec().evaluate_operation_with_burn_clear(&bound).unwrap();
    let assignment = realization::RelationId::new(
        OperationId::Clear,
        RelationKind::ClearAssignment,
        RelationSubject::StateField {
            field: StateField::YL,
        },
    );
    let floor_id = realization::RelationId::new(
        OperationId::Clear,
        RelationKind::ClearFloor,
        RelationSubject::Operation,
    );
    assert_eq!(
        report.verdict(&assignment).unwrap().status,
        RelationStatus::Failed {
            reason: realization::RelationFailure::BurnClearLaw
        }
    );
    assert!(matches!(
        &report.verdict(&floor_id).unwrap().status,
        RelationStatus::Blocked { prerequisites } if prerequisites.contains(&assignment)
    ));
    assert_eq!(
        realization::validate_state_succession(&bound, &projected.state_succession),
        Err(BurnClearRefusal::StateSuccession)
    );
}

#[test]
fn clear_copied_fields_refuse_changes() {
    let projected = execute(
        &super::ash_clear_tests::world_with_two_ash(),
        FeeEnvelope::default(),
    );
    assert_agrees(&projected);
    for field in [
        StateField::Omega,
        StateField::YT,
        StateField::Q,
        StateField::Cycle,
        StateField::Maturity,
    ] {
        let mut changed = projected.extension.clone();
        if let BurnClearPayload::Clear { successor, .. } = &mut changed.payload {
            let metadata = &mut successor.metadata;
            match field {
                StateField::Omega => {
                    metadata.omega =
                        realization::ProtocolAmount::new(metadata.omega.get() + 1).unwrap();
                }
                StateField::YT => {
                    metadata.y_t =
                        realization::ProtocolAmount::new(metadata.y_t.get() + 1).unwrap();
                }
                StateField::Q => {
                    metadata.q = realization::ProtocolAmount::new(metadata.q.get() + 1).unwrap();
                }
                StateField::Cycle => {
                    metadata.cycle = realization::Cycle::new(metadata.cycle.get() + 1);
                }
                StateField::Maturity => metadata.maturity = realization::Maturity::Complete,
                StateField::YL => unreachable!(),
            }
        }
        let bound = realization::validate_burn_clear_observation(
            projected.common.observation(),
            &projected.extension.context,
            Some(&changed),
        )
        .unwrap();
        assert_eq!(
            realization::validate_state_succession(&bound, &projected.state_succession),
            Err(BurnClearRefusal::StateSuccession)
        );
        let result = spec().evaluate_operation_with_burn_clear(&bound).unwrap();
        let id = realization::RelationId::new(
            OperationId::Clear,
            RelationKind::ClearAssignment,
            RelationSubject::StateField { field },
        );
        assert!(matches!(
            result.verdict(&id).unwrap().status,
            RelationStatus::Failed { .. }
        ));
    }
}

#[test]
fn clear_destruction_coordinates_refuse_changes() {
    let world = super::ash_clear_tests::world_with_two_ash();
    let execution = execute_bound(
        &world,
        ClearAsh {
            ash_inputs: find_ash(&world),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    )
    .unwrap();
    let projected = observe_clear(&execution).unwrap();
    assert_agrees(&projected);
    for coordinate in 0..4 {
        let mut changed = projected.extension.clone();
        if let BurnClearPayload::Clear { destruction, .. } = &mut changed.payload {
            match coordinate {
                0 => destruction.amount = realization::ProtocolAmount::ONE,
                1 => destruction.tag = architecture::TagId::Burn,
                2 => destruction.asset = architecture::AssetId::Lbtc,
                _ => destruction.flow = None,
            }
        }
        let actual = realization::validate_burn_clear_observation(
            projected.common.observation(),
            &projected.extension.context,
            Some(&changed),
        );
        assert_eq!(
            actual.unwrap_err(),
            if coordinate == 3 {
                BurnClearRefusal::DataAssociation
            } else {
                BurnClearRefusal::DestructionBinding
            }
        );
    }
    let mut common = projected.common.observation().clone();
    let mut leg = common.region.canonical_partition.flows[0].destructions[0];
    leg.tag = architecture::TagId::Burn;
    common.region.canonical_partition.flows[0]
        .destructions
        .push(leg);
    let mut extension = projected.extension;
    extension.context = context_for(&execution, common.clone());
    assert_eq!(
        realization::validate_burn_clear_observation(&common, &extension.context, Some(&extension)),
        Err(BurnClearRefusal::DestructionBinding)
    );
}

#[test]
fn clear_residual_coordinates_refuse_changes() {
    for world in [
        super::ash_clear_tests::world_with_two_ash(),
        partial_world(),
    ] {
        let projected = execute(&world, FeeEnvelope::default());
        assert_agrees(&projected);
        let mut event = projected.event.clone();
        if event.residual.is_empty() {
            event.residual.push((
                projected.event.ash_sources[0],
                realization::ProtocolAmount::ONE,
            ));
        } else {
            event.residual[0].1 = realization::ProtocolAmount::new(2).unwrap();
        }
        assert_eq!(
            realization::validate_clear_event(&bind(&projected), &event),
            Err(BurnClearRefusal::ClearEvent)
        );
        if !projected.event.residual.is_empty() {
            let mut count = projected.event.clone();
            count.residual.push(count.residual[0]);
            assert_eq!(
                realization::validate_clear_event(&bind(&projected), &count),
                Err(BurnClearRefusal::ClearEvent)
            );
        }
        let mut accounting = projected.accounting.clone();
        if let AccountingValues::Clear { residual, .. } = &mut accounting.values {
            *residual = realization::ProtocolAmount::new(2).unwrap();
        }
        assert_eq!(
            realization::validate_accounting(&bind(&projected), &accounting),
            Err(BurnClearRefusal::Accounting)
        );
    }
}

#[test]
fn clear_context_and_state_association_refuse_changes() {
    let world = super::ash_clear_tests::world_with_two_ash();
    let execution = execute_bound(
        &world,
        ClearAsh {
            ash_inputs: find_ash(&world),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    )
    .unwrap();
    let projected = observe_clear(&execution).unwrap();
    assert_agrees(&projected);
    let mut changed = projected.common.observation().clone();
    changed.bounds.remove(&BoundId::AshBatchMax);
    assert_eq!(
        realization::validate_burn_clear_observation(
            &changed,
            &projected.extension.context,
            Some(&projected.extension)
        ),
        Err(BurnClearRefusal::CommonObservation)
    );
    let mut foreign = projected.extension.clone();
    foreign.context = realization::BurnClearContext::bind_observation(
        projected.common.observation().clone(),
        [1; 32],
        [2; 32],
    )
    .unwrap();
    assert_eq!(
        realization::validate_burn_clear_observation(
            projected.common.observation(),
            &projected.extension.context,
            Some(&foreign)
        ),
        Err(BurnClearRefusal::Transaction)
    );
    let mut wrong_certificate = projected.extension.clone();
    wrong_certificate.context = realization::BurnClearContext::bind_observation(
        projected.common.observation().clone(),
        execution.certificate().txid.0,
        [3; 32],
    )
    .unwrap();
    assert_eq!(
        realization::validate_burn_clear_observation(
            projected.common.observation(),
            &projected.extension.context,
            Some(&wrong_certificate)
        ),
        Err(BurnClearRefusal::Certificate)
    );
    let mut state = projected.extension.clone();
    if let BurnClearPayload::Clear { successor, .. } = &mut state.payload {
        successor.reference = projected.state_succession.predecessor.reference;
    }
    assert_eq!(
        realization::validate_burn_clear_observation(
            projected.common.observation(),
            &projected.extension.context,
            Some(&state)
        ),
        Err(BurnClearRefusal::StateBinding)
    );
    let mut edge = projected.state_succession.clone();
    edge.successor.reference = edge.predecessor.reference;
    assert_eq!(
        realization::validate_state_succession(&bind(&projected), &edge),
        Err(BurnClearRefusal::StateSuccession)
    );
}

#[test]
fn clear_projection_roles_are_independent() {
    let projected = execute(
        &super::ash_clear_tests::world_with_two_ash(),
        FeeEnvelope::default(),
    );
    let bound = bind(&projected);
    assert_agrees(&projected);
    let mut event = projected.event.clone();
    event.omega = realization::ProtocolAmount::ONE;
    assert_eq!(
        realization::validate_clear_event(&bound, &event),
        Err(BurnClearRefusal::ClearEvent)
    );
    realization::validate_accounting(&bound, &projected.accounting).unwrap();
    realization::validate_state_succession(&bound, &projected.state_succession).unwrap();
    let mut accounting = projected.accounting.clone();
    accounting.event = None;
    assert_eq!(
        realization::validate_accounting(&bound, &accounting),
        Err(BurnClearRefusal::Accounting)
    );
    realization::validate_clear_event(&bound, &projected.event).unwrap();
    let mut state = projected.state_succession.clone();
    state.predecessor = state.successor;
    assert_eq!(
        realization::validate_state_succession(&bound, &state),
        Err(BurnClearRefusal::StateSuccession)
    );
    assert!(unresolved_model_evidence(&report(&projected), &projected.common).is_empty());
    assert!(!report(&projected).is_evidence_complete());
}

#[test]
fn clear_entry_bounds_and_scope_are_explicit() {
    let world = super::ash_clear_tests::world_with_two_ash();
    let execution = execute_bound(
        &world,
        ClearAsh {
            ash_inputs: find_ash(&world),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    )
    .unwrap();
    let projected = observe_clear(&execution).unwrap();
    assert_agrees(&projected);
    for missing in [BoundId::AshBatchMax, BoundId::FeeSponsorInputMax] {
        let mut common = projected.common.observation().clone();
        common.bounds.remove(&missing);
        let bound = rebound(&projected, common, &execution);
        assert_eq!(
            spec().evaluate_operation_with_burn_clear(&bound),
            Err(realization::RealizationError::MissingBoundValue(missing))
        );
        if missing == BoundId::AshBatchMax {
            let context = bound.read_observation().context.clone();
            let mut event = projected.event.clone();
            event.context = context.clone();
            let mut succession = projected.state_succession.clone();
            succession.context = context.clone();
            let mut accounting = projected.accounting.clone();
            accounting.context = context;
            assert_eq!(
                realization::validate_clear_event(&bound, &event),
                Err(BurnClearRefusal::ClearTransition)
            );
            assert_eq!(
                realization::validate_state_succession(&bound, &succession),
                Err(BurnClearRefusal::ClearTransition)
            );
            assert_eq!(
                realization::validate_accounting(&bound, &accounting),
                Err(BurnClearRefusal::ClearTransition)
            );
        }
    }
    assert_eq!(
        spec().evaluate_operation(projected.common.observation()),
        Err(realization::RealizationError::BurnClearObservation(
            BurnClearRefusal::MissingExtension
        ))
    );
    let pilot = realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::phase1_pilots(),
    )
    .unwrap();
    assert_eq!(
        pilot.evaluate_operation_with_burn_clear(&bind(&projected)),
        Err(realization::RealizationError::OperationOutsideScope(
            OperationId::Clear
        ))
    );
}

#[test]
fn clear_model_refusal_is_not_an_observation() {
    let world = partial_world();
    let first = execute_bound(
        &world,
        ClearAsh {
            ash_inputs: find_ash(&world),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    )
    .unwrap();
    let projected = observe_clear(&first).unwrap();
    assert_agrees(&projected);
    let after = first.after();
    assert_eq!(
        execute_bound(
            after,
            ClearAsh {
                ash_inputs: find_ash(after),
                fee_envelope: FeeEnvelope::default()
            },
            next_order(after)
        )
        .unwrap_err(),
        Guard::ZeroProgress
    );
}
