//! Residue noninterference over branch-local computation, exhaustive
//! over the operation registry.
//!
//! Implements `´test:verification:residue-noninterference-branches´`:
//! audit-only residue history must be unreadable by every covenant
//! operation. The dispatch below matches `OperationId` without a
//! wildcard, so declaring a new operation fails compilation here until
//! it supplies a noninterference scenario, and the census test drives
//! the dispatch from `ARCHITECTURE.operations`, so no declared
//! operation can be skipped. Coverage is mechanical, not
//! representative.

use architecture::{ARCHITECTURE, ids::OperationId};

use super::advanced_fixtures::*;
use super::distribution_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

/// Apply `transition` on `world` twice — once with a perturbed residue
/// projection in the history — and require identical branch-local
/// outcomes.
fn check<T: Transition>(world: &World, transition: &T) {
    assert_protocol_noninterference(world, transition, next_order(world), |history| {
        perturb_residue_projection(history, sat(7), sat(11));
    })
    .unwrap();
}

/// Two separately burned ash objects, so compaction and clear have
/// more than one input to consume.
fn world_with_two_ash() -> World {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));
    let first = burn_from_owner(&world, ALICE, sat(50), Vec::new());
    let with_bob = give_live_receipt(&first, BOB, sat(50));
    burn_from_owner(&with_bob, BOB, sat(50), Vec::new())
}

/// One applicable transition per declared operation, checked for
/// branch-local residue noninterference. The match is deliberately
/// wildcard-free.
#[allow(clippy::too_many_lines)]
fn run_noninterference(operation: OperationId) {
    match operation {
        OperationId::CreateRequest => {
            let world = test_fixtures::world();
            let gross = sat(1_000);
            let (funded, funding_input) = fund_lbtc(&world, ALICE, gross);

            check(
                &funded,
                &CreateRequest {
                    funder: ALICE,
                    funding_inputs: vec![funding_input],
                    signers: signers(&[ALICE]),
                    refund_key: ALICE,
                    receipt_owner: BOB,
                    deposit_principal: sat(900),
                    gross_value: gross,
                    change: None,
                    chain_fee: Sat::ZERO,
                },
            );
        }

        OperationId::CancelRequest => {
            let world = create_request_for(&test_fixtures::world(), ALICE, BOB, sat(900), sat(100));
            let request = find_request(&world, BOB);

            check(
                &world,
                &CancelRequest {
                    request,
                    signers: signers(&[ALICE]),
                    fee_envelope: FeeEnvelope::default(),
                },
            );
        }

        OperationId::AdmitDeposits => {
            let world = create_request_for(&test_fixtures::world(), ALICE, BOB, sat(900), sat(100));
            let request = find_request(&world, BOB);

            check(
                &world,
                &AdmitDeposits {
                    requests: vec![request],
                    admission_reward: sat(40),
                    reward_owner: RELAYER,
                },
            );
        }

        OperationId::Cycle => {
            let world = advance_blocks(&test_fixtures::world(), 100);

            check(
                &world,
                &RunCycle {
                    caller: CycleCaller::Anyone,
                    operator_signers: SignerSet::new(),
                    fee_envelope: FeeEnvelope::default(),
                },
            );
        }

        OperationId::SettleDistribution => {
            let (world, cycle) = world_with_distribution(&[(ALICE, ALICE, 400), (BOB, BOB, 600)]);
            let control = find_distribution_control(&world, cycle);
            let vault = find_distribution_vault(&world, cycle);
            let entitlement = find_entitlement(&world, ALICE);

            check(
                &world,
                &SettleDistribution {
                    control,
                    vault,
                    entitlements: vec![entitlement],
                    fee_envelope: FeeEnvelope::default(),
                },
            );
        }

        OperationId::TransferLive => {
            let world = test_fixtures::world();
            let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];
            let value = world.utxo(input).unwrap().value;
            let first = sat(value.get() / 2);
            let second = value.checked_sub(first).unwrap();

            check(
                &world,
                &TransferReceipts {
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
                },
            );
        }

        OperationId::TransferTimeLocked => {
            let world = test_fixtures::world();
            let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked)[0];
            let value = world.utxo(input).unwrap().value;
            let first = sat(value.get() / 3);
            let second = value.checked_sub(first).unwrap();

            check(
                &world,
                &TransferReceipts {
                    class: ReceiptClass::TimeLocked,
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
                },
            );
        }

        OperationId::Redeem => {
            let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));
            let receipt = find_receipts(&world, ALICE, ReceiptClass::Live)[0];

            check(
                &world,
                &RedeemReceipt {
                    receipt,
                    signers: signers(&[ALICE]),
                    fee_envelope: FeeEnvelope::default(),
                },
            );
        }

        OperationId::ReceiptRelabel => {
            let world = mature_world(&test_fixtures::world());
            let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked)[0];

            check(
                &world,
                &RelabelReceipts {
                    receipts: vec![input],
                    fee_envelope: FeeEnvelope::default(),
                },
            );
        }

        OperationId::Burn => {
            let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));
            let receipt = find_receipts(&world, ALICE, ReceiptClass::Live)[0];

            check(
                &world,
                &BurnReceipts {
                    receipts: vec![receipt],
                    signers: signers(&[ALICE]),
                    ash_value: sat(100),
                    change: Vec::new(),
                    records: vec![BurnRecord {
                        record_index: 0,
                        address: ADDRESS_A,
                        amount: sat(100),
                    }],
                    fee_envelope: FeeEnvelope::default(),
                },
            );
        }

        OperationId::CompactAsh => {
            let world = world_with_two_ash();
            let ash = find_ash(&world);

            check(
                &world,
                &CompactAsh {
                    ash_inputs: ash,
                    fee_envelope: FeeEnvelope::default(),
                },
            );
        }

        OperationId::Clear => {
            let world = world_with_two_ash();
            let ash = find_ash(&world);

            check(
                &world,
                &ClearAsh {
                    ash_inputs: ash,
                    fee_envelope: FeeEnvelope::default(),
                },
            );
        }

        OperationId::AnnounceMaturity => {
            let world = test_fixtures::world();

            check(
                &world,
                &AnnounceMaturity {
                    maturity_cycle: 10,
                    signers: signers(&[OPERATOR_KEY]),
                    fee_envelope: FeeEnvelope::default(),
                },
            );
        }
    }
}

/// The census: every operation declared in the manifest is checked.
/// Together with the wildcard-free dispatch, this makes the residue
/// noninterference claim mechanically exhaustive over the registry.
#[test]
fn every_declared_operation_is_residue_noninterfering() {
    for operation in ARCHITECTURE.operations {
        run_noninterference(operation.id);
    }
}

// Per-operation entry points for failure attribution.

#[test]
fn create_request_does_not_read_residue_history() {
    run_noninterference(OperationId::CreateRequest);
}

#[test]
fn cancel_request_does_not_read_residue_history() {
    run_noninterference(OperationId::CancelRequest);
}

#[test]
fn admit_deposits_does_not_read_residue_history() {
    run_noninterference(OperationId::AdmitDeposits);
}

#[test]
fn cycle_does_not_read_residue_history() {
    run_noninterference(OperationId::Cycle);
}

#[test]
fn settle_distribution_does_not_read_residue_history() {
    run_noninterference(OperationId::SettleDistribution);
}

#[test]
fn transfer_live_does_not_read_residue_history() {
    run_noninterference(OperationId::TransferLive);
}

#[test]
fn transfer_time_locked_does_not_read_residue_history() {
    run_noninterference(OperationId::TransferTimeLocked);
}

#[test]
fn redeem_does_not_read_residue_history() {
    run_noninterference(OperationId::Redeem);
}

#[test]
fn receipt_relabel_does_not_read_residue_history() {
    run_noninterference(OperationId::ReceiptRelabel);
}

#[test]
fn burn_does_not_read_residue_history() {
    run_noninterference(OperationId::Burn);
}

#[test]
fn compact_ash_does_not_read_residue_history() {
    run_noninterference(OperationId::CompactAsh);
}

#[test]
fn clear_does_not_read_residue_history() {
    run_noninterference(OperationId::Clear);
}

#[test]
fn announce_maturity_does_not_read_residue_history() {
    run_noninterference(OperationId::AnnounceMaturity);
}
