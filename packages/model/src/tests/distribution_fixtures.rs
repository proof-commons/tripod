//! Scenario helpers for distributions and maturity.
//!
//! Implements `´test:verification:distribution-fixtures´`.

use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

pub fn world_with_requests(requests: &[(OwnerKey, OwnerKey, u64)]) -> World {
    let mut world = test_fixtures::world();

    for (funder, owner, principal) in requests {
        world = create_request_for(&world, *funder, *owner, sat(*principal), Sat::ONE);
    }

    admit_all_requests(&world)
}

pub fn world_with_distribution(requests: &[(OwnerKey, OwnerKey, u64)]) -> (World, Cycle) {
    let admitted = world_with_requests(requests);

    let cycled = run_forced_cycle(&admitted);

    let cycle = cycled.state().unwrap().1.cycle;

    (cycled, cycle)
}

pub fn settle_batch(world: &World, cycle: Cycle, entitlements: Vec<OutPoint>) -> World {
    let control = find_distribution_control(world, cycle);

    let vault = find_distribution_vault(world, cycle);

    SettleDistribution {
        control,
        vault,
        entitlements,
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_order(world))
    .unwrap()
}

pub fn find_all_entitlements_for_cycle(world: &World, cycle: Cycle) -> Vec<OutPoint> {
    world
        .utxos
        .iter()
        .filter_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (Asset::Ent, Meta::DepositEntitlement { target_cycle, .. })
                if target_cycle == cycle =>
            {
                Some(*outpoint)
            }

            _ => None,
        })
        .collect()
}

pub fn announce_at_minimum_lead(world: &World) -> (World, Cycle) {
    let state = world.state().unwrap().1;

    let maturity_cycle = state
        .cycle
        .checked_add(world.constants.min_maturity_lead)
        .unwrap();

    let announced = AnnounceMaturity {
        maturity_cycle,
        signers: signers(&[OPERATOR_KEY]),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_order(world))
    .unwrap();

    (announced, maturity_cycle)
}

pub fn advance_to_cycle(world: &World, target_cycle: Cycle) -> World {
    let mut current = world.clone();

    while current.state().unwrap().1.cycle < target_cycle {
        current = advance_blocks(&current, current.constants.max_cadence_blocks);

        current = RunCycle {
            caller: CycleCaller::Anyone,
            operator_signers: SignerSet::new(),
            fee_envelope: FeeEnvelope::default(),
        }
        .apply(&current, next_order(&current))
        .unwrap();
    }

    current
}

pub fn mature_world(world: &World) -> World {
    let (announced, maturity_cycle) = announce_at_minimum_lead(world);

    advance_to_cycle(&announced, maturity_cycle)
}
