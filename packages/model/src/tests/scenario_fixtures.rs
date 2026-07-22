//! Deterministic scenario-test support.
//!
//! Implements `´test:verification:scenario-fixtures´`.
//!
//! The fixtures use zero-fee transactions in the abstract model unless
//! a test specifically exercises sponsorship.

use crate::*;

pub const ALICE: OwnerKey = OwnerKey([1_u8; 32]);

pub const BOB: OwnerKey = OwnerKey([2_u8; 32]);

pub const CAROL: OwnerKey = OwnerKey([3_u8; 32]);

pub const RELAYER: OwnerKey = OwnerKey([4_u8; 32]);

pub const SPONSOR: OwnerKey = OwnerKey([5_u8; 32]);

pub const ADDRESS_A: AttestationAddress = AttestationAddress([11_u8; 32]);

pub const ADDRESS_B: AttestationAddress = AttestationAddress([12_u8; 32]);

pub fn sat(value: u64) -> Sat {
    Sat::new(value).unwrap()
}

pub fn signers(owners: &[OwnerKey]) -> SignerSet {
    owners.iter().copied().collect()
}

pub fn next_order(world: &World) -> CanonicalOrder {
    let previous = world.history.last_order();

    CanonicalOrder {
        height: previous.height,
        tx_index: previous.tx_index.checked_add(1).expect("test tx index"),
    }
}

pub fn next_block_order(world: &World) -> CanonicalOrder {
    let previous = world.history.last_order();

    CanonicalOrder {
        height: previous.height.checked_add(1).expect("test block height"),
        tx_index: 0,
    }
}

/// Apply a transition through the normative [`execute`] path, which
/// enforces the global invariant wrapper, and return the successor world.
///
/// Positive scenario tests should prefer this over a bare
/// `transition.apply(..).unwrap()` followed by a separate
/// `check_invariant`: it makes the evidence class explicit — the world
/// advanced through the invariant-wrapped public path, not merely the
/// operation guard. Fault, authorization-failure, corruption, and
/// kernel/branch-local tests deliberately keep raw `apply`.
pub fn apply_checked<T: Transition>(world: &World, transition: &T, order: CanonicalOrder) -> World {
    execute(world, transition, order).expect("checked transition")
}

pub fn advance_blocks(world: &World, blocks: u64) -> World {
    let mut next = world.clone();

    next.pace_age_blocks = next
        .pace_age_blocks
        .checked_add(blocks)
        .expect("test pace age");

    next
}

pub fn fund_lbtc(world: &World, owner: OwnerKey, value: Sat) -> (World, OutPoint) {
    let mut funded = world.clone();

    funded.adversary.lbtc = funded.adversary.lbtc.checked_add(value).unwrap();

    let outpoint = funded.next_outpoint;

    let injected =
        inject_open_object(&funded, Asset::Lbtc, value, Meta::PlainLbtc { owner }).unwrap();

    (injected, outpoint)
}

pub fn fee_envelope_exact(
    input: OutPoint,
    owner: OwnerKey,
    input_value: Sat,
    fee: Sat,
) -> FeeEnvelope {
    let change_value = input_value.checked_sub(fee).unwrap();

    FeeEnvelope {
        inputs: vec![input],

        signers: signers(&[owner]),

        chain_fee: fee,

        change: if change_value.is_zero() {
            None
        } else {
            Some(FeeChange {
                owner,
                value: change_value,
            })
        },
    }
}

pub fn empty_fee_envelope() -> FeeEnvelope {
    FeeEnvelope::default()
}

pub fn find_request(world: &World, owner: OwnerKey) -> OutPoint {
    world
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| match utxo.meta {
            Meta::DepositRequest { receipt_owner, .. } if receipt_owner == owner => Some(*outpoint),

            _ => None,
        })
        .expect("request")
}

pub fn find_entitlement(world: &World, owner: OwnerKey) -> OutPoint {
    world
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (
                Asset::Ent,
                Meta::DepositEntitlement {
                    owner: entitlement_owner,
                    ..
                },
            ) if entitlement_owner == owner => Some(*outpoint),

            _ => None,
        })
        .expect("entitlement")
}

pub fn find_distribution_control(world: &World, cycle: Cycle) -> OutPoint {
    world
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (
                Asset::DistCtl,
                Meta::DistributionControl {
                    cycle: control_cycle,
                    ..
                },
            ) if control_cycle == cycle => Some(*outpoint),

            _ => None,
        })
        .expect("distribution control")
}

pub fn find_distribution_vault(world: &World, cycle: Cycle) -> Option<OutPoint> {
    world
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (Asset::U, Meta::DistributionVault { cycle: vault_cycle }) if vault_cycle == cycle => {
                Some(*outpoint)
            }

            _ => None,
        })
}

pub fn find_receipts(world: &World, owner: OwnerKey, class: ReceiptClass) -> Vec<OutPoint> {
    world
        .utxos
        .iter()
        .filter_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (
                Asset::U,
                Meta::Receipt {
                    owner: receipt_owner,
                    class: receipt_class,
                },
            ) if receipt_owner == owner && receipt_class == class => Some(*outpoint),

            _ => None,
        })
        .collect()
}

pub fn find_ash(world: &World) -> Vec<OutPoint> {
    world
        .utxos
        .iter()
        .filter_map(|(outpoint, utxo)| {
            if matches!((utxo.asset, utxo.meta), (Asset::U, Meta::Ash)) {
                Some(*outpoint)
            } else {
                None
            }
        })
        .collect()
}

pub fn create_request_for(
    world: &World,
    funder: OwnerKey,
    owner: OwnerKey,
    principal: Sat,
    budget: Sat,
) -> World {
    let gross = principal.checked_add(budget).unwrap();

    let (funded, funding_input) = fund_lbtc(world, funder, gross);

    CreateRequest {
        funder,
        funding_inputs: vec![funding_input],
        signers: signers(&[funder]),

        refund_key: funder,
        receipt_owner: owner,

        deposit_principal: principal,
        gross_value: gross,

        change: None,
        chain_fee: Sat::ZERO,
    }
    .apply(&funded, next_order(&funded))
    .unwrap()
}

pub fn admit_all_requests(world: &World) -> World {
    let requests = world
        .utxos
        .iter()
        .filter_map(|(outpoint, utxo)| {
            if matches!(
                (utxo.asset, utxo.meta),
                (Asset::Lbtc, Meta::DepositRequest { .. }),
            ) {
                Some(*outpoint)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    AdmitDeposits {
        requests,
        admission_reward: Sat::ZERO,
        reward_owner: RELAYER,
    }
    .apply(world, next_order(world))
    .unwrap()
}

pub fn run_forced_cycle(world: &World) -> World {
    let aged = advance_blocks(world, world.constants.max_cadence_blocks);

    RunCycle {
        caller: CycleCaller::Anyone,

        operator_signers: SignerSet::new(),

        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&aged, next_order(&aged))
    .unwrap()
}
