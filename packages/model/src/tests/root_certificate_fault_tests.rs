//! Root-certificate corruption helpers and fault tests.
//!
//! Implements `´test:verification:certificate-corruption-fixtures´`
//! and `´test:verification:root-certificate-faults´`.

use super::scenario_fixtures::*;
use super::test_fixtures;
use super::test_fixtures::inject_and_get_outpoint;
use crate::*;

pub fn mutate_last_certificate(
    world: &World,
    mutation: impl FnOnce(&mut TransitionCertificate),
) -> World {
    let mut corrupted = world.clone();

    let certificate = corrupted
        .history
        .transitions
        .last_mut()
        .expect("transition certificate");

    mutation(certificate);

    corrupted
}

#[test]
fn wrong_state_successor_in_history_is_rejected() {
    let world = create_request_for(&test_fixtures::world(), ALICE, ALICE, sat(100), Sat::ONE);

    let admitted = admit_all_requests(&world);

    let corrupted = mutate_last_certificate(&admitted, |certificate| {
        if let Some(RootEdge::Succ { input, output: _ }) = certificate.state_edge {
            certificate.state_edge = Some(RootEdge::Succ {
                input,
                output: input,
            });
        }
    });

    assert!(matches!(
        check_invariant(&corrupted),
        Err(InvariantError::StateSuccession | InvariantError::HistoryProjection),
    ));
}

#[test]
fn missing_entitlement_authority_edge_is_rejected() {
    let world = create_request_for(&test_fixtures::world(), ALICE, ALICE, sat(100), Sat::ONE);

    let admitted = admit_all_requests(&world);

    let corrupted = mutate_last_certificate(&admitted, |certificate| {
        certificate.entitlement_authority_edge = None;
    });

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::StateSuccession),
    );
}

#[test]
fn invalid_resv_termination_on_operational_state_is_rejected() {
    let world = create_request_for(&test_fixtures::world(), ALICE, ALICE, sat(100), Sat::ONE);

    let admitted = admit_all_requests(&world);

    let corrupted = mutate_last_certificate(&admitted, |certificate| {
        if let Some(RootEdge::Succ { input, .. }) = certificate.resv_edge {
            certificate.resv_edge = Some(RootEdge::Term { input });
        }
    });

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::ResvSuccession),
    );
}

#[test]
fn cursor_relinked_to_decoy_is_rejected() {
    let mut world = test_fixtures::world();

    world.adversary.lbtc = sat(2_000_000);

    let active_value = world.active_resv().unwrap().1.value;

    let (mut corrupted, decoy) =
        inject_and_get_outpoint(&world, Asset::Lbtc, active_value, Meta::Resv);

    corrupted.roots.resv = Some(decoy);

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::ResvSuccession),
    );
}

#[test]
fn intermediate_state_edge_input_corruption_is_rejected() {
    let created = create_request_for(&test_fixtures::world(), ALICE, ALICE, sat(100), Sat::ONE);

    let admitted = admit_all_requests(&created);

    // A later STATE-advancing transition restores the expected final
    // cursor; only full replay detects the intermediate corruption.
    let announced = AnnounceMaturity {
        maturity_cycle: admitted.state().unwrap().1.cycle + admitted.constants.min_maturity_lead,
        signers: signers(&[OPERATOR_KEY]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&admitted, next_order(&admitted))
    .unwrap();

    let mut corrupted = announced;

    let first_state_transition = corrupted
        .history
        .transitions
        .iter_mut()
        .find(|certificate| certificate.state_edge.is_some())
        .unwrap();

    if let Some(RootEdge::Succ { output, .. }) = first_state_transition.state_edge {
        first_state_transition.state_edge = Some(RootEdge::Succ {
            input: output,
            output,
        });
    }

    assert!(matches!(
        check_invariant(&corrupted),
        Err(InvariantError::StateSuccession | InvariantError::HistoryProjection),
    ));
}

#[test]
fn intermediate_resv_edge_input_corruption_is_rejected() {
    let first = create_request_for(&test_fixtures::world(), ALICE, ALICE, sat(100), Sat::ONE);

    let admitted = admit_all_requests(&first);

    // The cycle advances RESV again, restoring the final cursor.
    let cycled = run_forced_cycle(&admitted);

    let mut corrupted = cycled;

    let first_resv_transition = corrupted
        .history
        .transitions
        .iter_mut()
        .find(|certificate| certificate.resv_edge.is_some())
        .unwrap();

    if let Some(RootEdge::Succ { output, .. }) = first_resv_transition.resv_edge {
        first_resv_transition.resv_edge = Some(RootEdge::Succ {
            input: output,
            output,
        });
    }

    assert!(matches!(
        check_invariant(&corrupted),
        Err(InvariantError::ResvSuccession | InvariantError::HistoryProjection),
    ));
}

#[test]
fn intermediate_pace_edge_corruption_is_rejected() {
    let first = run_forced_cycle(&test_fixtures::world());

    // The second cycle advances PACE again, restoring the final cursor.
    let second = run_forced_cycle(&first);

    let mut corrupted = second;

    let first_pace_transition = corrupted
        .history
        .transitions
        .iter_mut()
        .find(|certificate| certificate.pace_edge.is_some())
        .unwrap();

    if let Some(RootEdge::Succ { output, .. }) = first_pace_transition.pace_edge {
        first_pace_transition.pace_edge = Some(RootEdge::Succ {
            input: output,
            output,
        });
    }

    assert!(matches!(
        check_invariant(&corrupted),
        Err(InvariantError::StateSuccession | InvariantError::HistoryProjection),
    ));
}

#[test]
fn certificate_claiming_a_genesis_receipt_outpoint_is_rejected() {
    // The creation census seeds every genesis-created outpoint, not
    // only the five roots: a corrupted certificate claiming a genesis
    // receipt as newly created is a duplicate creation.
    let mut corrupted = run_forced_cycle(&test_fixtures::world());

    let genesis_receipt = corrupted.history.genesis.live_receipt_out;

    corrupted
        .history
        .transitions
        .last_mut()
        .unwrap()
        .created
        .insert(genesis_receipt);

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::HistoryProjection),
    );
}

#[test]
fn duplicate_transition_txid_is_rejected() {
    let first = run_forced_cycle(&test_fixtures::world());

    let second = run_forced_cycle(&first);

    let mut corrupted = second;

    let first_txid = corrupted.history.transitions[0].txid;

    corrupted.history.transitions[1].txid = first_txid;

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::HistoryProjection),
    );
}

#[test]
fn non_increasing_transition_order_is_rejected() {
    let first = run_forced_cycle(&test_fixtures::world());

    let second = run_forced_cycle(&first);

    let mut corrupted = second;

    let first_order = corrupted.history.transitions[0].order;

    corrupted.history.transitions[1].order = first_order;

    assert_eq!(
        check_invariant(&corrupted),
        Err(InvariantError::HistoryProjection),
    );
}

#[test]
fn off_pool_branch_cannot_advance_pace() {
    let world = test_fixtures::world();

    let receipt = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];

    let transferred = TransferReceipts {
        class: ReceiptClass::Live,
        inputs: vec![receipt],
        outputs: vec![ReceiptDestination {
            owner: ALICE,
            value: world.utxo(receipt).unwrap().value,
        }],
        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    let mut corrupted = transferred;

    let certificate = corrupted.history.transitions.last_mut().unwrap();

    certificate.pace_edge = Some(RootEdge::Succ {
        input: world.roots.pace,
        output: world.roots.pace,
    });

    assert!(matches!(
        check_invariant(&corrupted),
        Err(InvariantError::StateSuccession | InvariantError::HistoryProjection),
    ));
}
