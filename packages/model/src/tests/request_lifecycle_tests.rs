//! Request-lifecycle tests.
//!
//! Implements `´test:verification:request-lifecycle´`.

use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn conforming_request_can_be_created() {
    let world = test_fixtures::world();

    let next = create_request_for(&world, ALICE, BOB, sat(900), sat(100));

    let request = find_request(&next, BOB);

    let view =
        validate_request_for_admission(&next.constants, next.utxo(request).unwrap()).unwrap();

    assert_eq!(view.deposit_principal, sat(900));

    assert_eq!(view.admission_budget, sat(100));

    check_invariant(&next).unwrap();
}

#[test]
fn malformed_open_request_is_inert_but_not_admissible() {
    let mut world = test_fixtures::world();

    world.adversary.lbtc = sat(100);

    let outpoint = world.next_outpoint;

    let next = inject_open_object(
        &world,
        Asset::Lbtc,
        sat(100),
        Meta::DepositRequest {
            pool_id: world.constants.pool_id,
            refund_key: ALICE,
            receipt_owner: BOB,
            deposit_principal: sat(100),
        },
    )
    .unwrap();

    check_invariant(&next).unwrap();

    assert_eq!(
        validate_request_for_admission(&next.constants, next.utxo(outpoint).unwrap()),
        Err(Guard::PartitionPin),
    );
}

#[test]
fn request_cancellation_refunds_full_request_value() {
    let world = create_request_for(&test_fixtures::world(), ALICE, BOB, sat(900), sat(100));

    let request = find_request(&world, BOB);

    let request_value = world.utxo(request).unwrap().value;

    let next = apply_checked(
        &world,
        &CancelRequest {
            request,
            signers: signers(&[ALICE]),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );

    let refund_total = next
        .utxos
        .values()
        .filter_map(|utxo| match (utxo.asset, utxo.meta) {
            (Asset::Lbtc, Meta::PlainLbtc { owner }) if owner == ALICE => Some(utxo.value),

            _ => None,
        })
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))
        .unwrap();

    assert!(refund_total >= request_value);

    assert!(!next.utxos.contains_key(&request));

    check_invariant(&next).unwrap();
}

#[test]
fn wrong_refund_signer_cannot_cancel() {
    let world = create_request_for(&test_fixtures::world(), ALICE, BOB, sat(900), sat(100));

    let request = find_request(&world, BOB);

    assert_eq!(
        CancelRequest {
            request,
            signers: signers(&[BOB]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::BadSignature),
    );
}

/// A hand-built cancellation whose protocol region refunds
/// `refund_value` and whose generic sponsor region returns
/// `change_value`.
///
/// The high-level constructor cannot express the vector below, because
/// it derives the refund from the consumed request. Building the
/// regions directly is what lets the test move one unit across the
/// sponsor boundary while leaving the whole-transaction L-BTC equation
/// intact.
fn cancellation_with_regions(
    world: &World,
    request: OutPoint,
    sponsor: OutPoint,
    refund_value: Sat,
    change_value: Sat,
) -> Result<crate::kernel::CommitResult, Guard> {
    use crate::kernel::{OpenFlow, TxBuilder};

    let mut tx = TxBuilder::new(world, BranchKind::CancelRequest, next_order(world));

    tx.consume(request)?;

    let refund = tx.emit(Asset::Lbtc, refund_value, Meta::PlainLbtc { owner: ALICE });

    tx.declare_open_flow(OpenFlow {
        kind: OpenFlowKind::RequestRefund,
        source_inputs: vec![request],
        destination_outputs: vec![refund],
        fee: Sat::ZERO,
    })?;

    tx.consume(sponsor)?;

    let change = tx.emit(
        Asset::Lbtc,
        change_value,
        Meta::PlainLbtc { owner: SPONSOR },
    );

    tx.declare_open_flow(OpenFlow {
        kind: OpenFlowKind::FeeSponsor,
        source_inputs: vec![sponsor],
        destination_outputs: vec![change],
        fee: sat(5),
    })?;

    tx.set_chain_fee(sat(5));

    tx.finish()
}

/// The balanced-theft shape at the kernel, on the one current
/// operation where ordinary L-BTC carries protocol value.
///
/// One unit is taken from the formula-bound refund and given to sponsor
/// change. The aggregate L-BTC equation still holds and every sponsor
/// amount stays strictly positive, so a sponsor-positivity check would
/// never fire — which is precisely why the O3 reduction set keeps this
/// vector and drops positivity. Per-region exactness rejects it: the
/// refund region has to balance on its own, and it no longer does.
#[test]
fn balanced_theft_across_the_sponsor_boundary_is_rejected() {
    let world = create_request_for(&test_fixtures::world(), ALICE, BOB, sat(900), sat(100));
    let request = find_request(&world, BOB);
    let request_value = world.utxo(request).unwrap().value;
    let (world, sponsor) = fund_lbtc(&world, SPONSOR, sat(20));

    cancellation_with_regions(&world, request, sponsor, request_value, sat(15))
        .expect("the exact regions balance");

    let stolen = request_value.checked_sub(sat(1)).unwrap();

    assert_eq!(
        cancellation_with_regions(&world, request, sponsor, stolen, sat(16)),
        Err(Guard::OpenFlowMismatch),
    );
}
