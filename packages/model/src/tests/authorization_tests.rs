//! Operation-authorization regression matrix.
//!
//! Implements `´test:verification:authorization-matrix´`.
//!
//! Evidence class: model-signer-set. `SignerSet` membership models
//! authorization of the complete transaction output set under an
//! output-committing sighash mode; exact signature opcodes and sighash
//! flags remain compiler/deployment obligations, per the generated
//! table `[tbl:manifest:operation-authorization-evidence]`.

use super::distribution_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

fn world_with_two_owner_live_receipts() -> (World, OutPoint, OutPoint) {
    let world = test_fixtures::world();

    let input = find_receipts(&world, GENESIS_OWNER, ReceiptClass::Live)[0];

    let value = world.utxo(input).unwrap().value;

    let alice_value = sat(value.get() / 2);

    let bob_value = value.checked_sub(alice_value).unwrap();

    let split = TransferReceipts {
        class: ReceiptClass::Live,
        inputs: vec![input],
        outputs: vec![
            ReceiptDestination {
                owner: ALICE,
                value: alice_value,
            },
            ReceiptDestination {
                owner: BOB,
                value: bob_value,
            },
        ],
        signers: signers(&[GENESIS_OWNER]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    let alice_receipt = find_receipts(&split, ALICE, ReceiptClass::Live)[0];
    let bob_receipt = find_receipts(&split, BOB, ReceiptClass::Live)[0];

    (split, alice_receipt, bob_receipt)
}

// Owner authorization: burn.

#[test]
fn mixed_owner_burn_requires_every_signer() {
    let (world, alice_receipt, bob_receipt) = world_with_two_owner_live_receipts();

    let total = world
        .utxo(alice_receipt)
        .unwrap()
        .value
        .checked_add(world.utxo(bob_receipt).unwrap().value)
        .unwrap();

    let burn = |signer_set: SignerSet| {
        BurnReceipts {
            receipts: vec![alice_receipt, bob_receipt],
            signers: signer_set,

            ash_value: total,
            change: Vec::new(),
            records: Vec::new(),

            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world))
    };

    assert_eq!(burn(signers(&[ALICE])), Err(Guard::BadSignature));
    assert_eq!(burn(signers(&[BOB])), Err(Guard::BadSignature));

    let next = burn(signers(&[ALICE, BOB])).unwrap();

    check_invariant(&next).unwrap();
}

// Owner authorization: transfer.

#[test]
fn mixed_owner_transfer_requires_every_signer() {
    let (world, alice_receipt, bob_receipt) = world_with_two_owner_live_receipts();

    let total = world
        .utxo(alice_receipt)
        .unwrap()
        .value
        .checked_add(world.utxo(bob_receipt).unwrap().value)
        .unwrap();

    let transfer = |signer_set: SignerSet| {
        TransferReceipts {
            class: ReceiptClass::Live,
            inputs: vec![alice_receipt, bob_receipt],
            outputs: vec![ReceiptDestination {
                owner: CAROL,
                value: total,
            }],
            signers: signer_set,
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world))
    };

    assert_eq!(transfer(signers(&[ALICE])), Err(Guard::BadSignature));

    let next = transfer(signers(&[ALICE, BOB])).unwrap();

    check_invariant(&next).unwrap();
}

// Owner authorization: redemption.

#[test]
fn redemption_with_wrong_signer_is_rejected() {
    let (world, alice_receipt, _bob_receipt) = world_with_two_owner_live_receipts();

    assert_eq!(
        RedeemReceipt {
            receipt: alice_receipt,
            signers: signers(&[BOB]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::BadSignature),
    );
}

// Refund-key authorization.

#[test]
fn cancellation_signed_by_receipt_owner_but_not_refund_key_is_rejected() {
    let world = test_fixtures::world();

    let principal = sat(100);
    let budget = Sat::ONE;
    let gross = principal.checked_add(budget).unwrap();

    let (funded, funding_input) = fund_lbtc(&world, CAROL, gross);

    // Refund key ALICE; receipt owner BOB.
    let requested = CreateRequest {
        funder: CAROL,
        funding_inputs: vec![funding_input],
        signers: signers(&[CAROL]),

        refund_key: ALICE,
        receipt_owner: BOB,

        deposit_principal: principal,
        gross_value: gross,

        change: None,
        chain_fee: Sat::ZERO,
    }
    .apply(&funded, next_order(&funded))
    .unwrap();

    let request = find_request(&requested, BOB);

    // The receipt owner signs, but the refund key does not.
    assert_eq!(
        CancelRequest {
            request,
            signers: signers(&[BOB]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&requested, next_order(&requested)),
        Err(Guard::BadSignature),
    );

    // The refund key alone is sufficient.
    let cancelled = CancelRequest {
        request,
        signers: signers(&[ALICE]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&requested, next_order(&requested))
    .unwrap();

    check_invariant(&cancelled).unwrap();
}

#[test]
fn request_funding_owned_by_someone_other_than_the_funder_is_rejected() {
    let world = test_fixtures::world();

    let principal = sat(100);
    let budget = Sat::ONE;
    let gross = principal.checked_add(budget).unwrap();

    // BOB owns and signs the funding input, but the request declares
    // ALICE as the funder: the declared funder is an enforced fact.
    let (funded, funding_input) = fund_lbtc(&world, BOB, gross);

    assert_eq!(
        CreateRequest {
            funder: ALICE,
            funding_inputs: vec![funding_input],
            signers: signers(&[BOB]),

            refund_key: ALICE,
            receipt_owner: ALICE,

            deposit_principal: principal,
            gross_value: gross,

            change: None,
            chain_fee: Sat::ZERO,
        }
        .apply(&funded, next_order(&funded)),
        Err(Guard::SponsorMismatch),
    );
}

// Operator authorization.

#[test]
fn operator_band_cycle_without_operator_signer_is_rejected() {
    let world = test_fixtures::world();

    // Inside the operator-only band: past minimum cadence, before
    // maximum cadence.
    let aged = advance_blocks(&world, world.constants.min_cadence_blocks);

    assert_eq!(
        RunCycle {
            caller: CycleCaller::Operator,
            operator_signers: SignerSet::new(),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&aged, next_order(&aged)),
        Err(Guard::BadSignature),
    );
}

#[test]
fn maturity_announcement_with_wrong_signer_is_rejected() {
    let world = test_fixtures::world();

    let maturity_cycle = world.constants.min_maturity_lead;

    assert_eq!(
        AnnounceMaturity {
            maturity_cycle,
            signers: signers(&[ALICE]),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&world, next_order(&world)),
        Err(Guard::BadSignature),
    );
}

// Permissionless branches accept empty signer sets. The operation
// types omit signer fields entirely (the strongest proof); these
// tests exercise the executable path end to end.

#[test]
fn permissionless_branches_accept_empty_signer_sets() {
    // Admission: permissionless.
    let world = test_fixtures::world();

    let funded = create_request_for(&world, ALICE, BOB, sat(500), Sat::ONE);

    let admitted = admit_all_requests(&funded);

    check_invariant(&admitted).unwrap();

    // Cadence-band cycle after maximum age: permissionless caller,
    // empty signer set.
    let aged = advance_blocks(&admitted, admitted.constants.max_cadence_blocks);

    let cycled = RunCycle {
        caller: CycleCaller::Anyone,
        operator_signers: SignerSet::new(),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&aged, next_order(&aged))
    .unwrap();

    check_invariant(&cycled).unwrap();

    // Settlement: permissionless.
    let cycle = cycled.state().unwrap().1.cycle;

    let entitlements = find_all_entitlements_for_cycle(&cycled, cycle);

    let settled = settle_batch(&cycled, cycle, entitlements);

    check_invariant(&settled).unwrap();

    // Burn to create ASH, then permissionless compaction and clear.
    let receipts = find_receipts(&settled, BOB, ReceiptClass::Live);

    assert_ne!(receipts, [] as [u64; 0]);

    let first_value = settled.utxo(receipts[0]).unwrap().value;

    let half = sat(first_value.get() / 2);

    let burned = BurnReceipts {
        receipts: vec![receipts[0]],
        signers: signers(&[BOB]),

        ash_value: half,
        change: vec![ReceiptDestination {
            owner: BOB,
            value: first_value.checked_sub(half).unwrap(),
        }],
        records: Vec::new(),

        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&settled, next_order(&settled))
    .unwrap();

    let cleared = ClearAsh {
        ash_inputs: find_ash(&burned),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&burned, next_order(&burned))
    .unwrap();

    check_invariant(&cleared).unwrap();
}

#[test]
fn permissionless_relabel_accepts_empty_signer_set() {
    let world = mature_world(&test_fixtures::world());

    let receipts = find_receipts(&world, GENESIS_OWNER, ReceiptClass::TimeLocked);

    assert_ne!(receipts, [] as [u64; 0]);

    let next = RelabelReceipts {
        receipts,
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&world, next_order(&world))
    .unwrap();

    check_invariant(&next).unwrap();
}

// Sponsor authorization.

#[test]
fn sponsor_input_owner_missing_from_signer_set_is_rejected() {
    let world = test_fixtures::world();

    let (funded, sponsor_input) = fund_lbtc(&world, SPONSOR, sat(10));

    // The envelope cites the sponsor input but omits the sponsor's
    // signature.
    let envelope = FeeEnvelope {
        inputs: vec![sponsor_input],
        signers: SignerSet::new(),
        chain_fee: sat(10),
        change: None,
    };

    assert_eq!(
        AnnounceMaturity {
            maturity_cycle: funded.constants.min_maturity_lead,
            signers: signers(&[OPERATOR_KEY]),
            fee_envelope: envelope,
        }
        .apply(&funded, next_order(&funded)),
        Err(Guard::BadSignature),
    );

    // The same envelope with the sponsor signature succeeds.
    let sponsored = AnnounceMaturity {
        maturity_cycle: funded.constants.min_maturity_lead,
        signers: signers(&[OPERATOR_KEY]),
        fee_envelope: fee_envelope_exact(sponsor_input, SPONSOR, sat(10), sat(10)),
    }
    .apply(&funded, next_order(&funded))
    .unwrap();

    check_invariant(&sponsored).unwrap();
}
