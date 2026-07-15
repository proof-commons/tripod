//! Checkpoint-reconstruction and forged-history rejection tests.
//!
//! `ReferenceIndexer::try_from(IndexerCheckpoint)` and
//! `ReferenceIndexer::from_history` are untrusted ingestion
//! boundaries: checkpoints and histories are publicly constructible,
//! so every constructor guarantee established by
//! `ValidatedChainView::new` must hold on these paths too. Each test
//! mutates exactly one fact a validated chain and kernel-derived
//! history could never produce and demonstrates closed failure.

use super::advanced_fixtures::*;
use super::scenario_fixtures::*;
use super::test_fixtures;
use super::test_fixtures::{block_hash, chain_view_for_history, txid};
use crate::ledger::serialize_query_unchecked_for_test;
use crate::*;

fn burned_world() -> World {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    burn_from_owner(
        &world,
        ALICE,
        sat(100),
        vec![BurnRecord {
            record_index: 0,
            address: ADDRESS_A,
            amount: sat(100),
        }],
    )
}

fn burned_indexer() -> ReferenceIndexer {
    let world = burned_world();

    let chain = chain_view_for_history(&world);

    ReferenceIndexer::from_history(&world.history, &chain, [0_u8; 32]).unwrap()
}

fn cleared_world() -> World {
    let world = give_live_receipt(&test_fixtures::world(), ALICE, sat(100));

    let burned = burn_from_owner(&world, ALICE, sat(50), Vec::new());

    let ash = find_ash(&burned);

    ClearAsh {
        ash_inputs: ash,
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(&burned, next_order(&burned))
    .unwrap()
}

fn from_forged_history(world: &World) -> Result<ReferenceIndexer, Guard> {
    let chain = chain_view_for_history(world);

    ReferenceIndexer::from_history(&world.history, &chain, [0_u8; 32])
}

// Checkpoint reconstruction: context identity.

#[test]
fn zero_network_id_checkpoint_is_rejected() {
    let mut checkpoint = burned_indexer().checkpoint();

    checkpoint.context.network_id = [0_u8; 32];

    assert_eq!(ReferenceIndexer::try_from(checkpoint), Err(Guard::Domain),);
}

#[test]
fn zero_genesis_id_checkpoint_is_rejected() {
    let mut checkpoint = burned_indexer().checkpoint();

    checkpoint.context.genesis_id = [0_u8; 32];

    assert_eq!(ReferenceIndexer::try_from(checkpoint), Err(Guard::Domain),);
}

#[test]
fn wrong_architecture_hash_checkpoint_is_rejected() {
    let mut checkpoint = burned_indexer().checkpoint();

    checkpoint.context.architecture_manifest_hash = [9_u8; 32];

    assert_eq!(
        ReferenceIndexer::try_from(checkpoint),
        Err(Guard::BadConstant),
    );
}

#[test]
fn unsupported_schema_checkpoint_is_rejected() {
    let mut checkpoint = burned_indexer().checkpoint();

    checkpoint.context.schema_version = ATTESTATION_SCHEMA_VERSION + 1;

    assert_eq!(
        ReferenceIndexer::try_from(checkpoint),
        Err(Guard::UnsupportedSchema),
    );
}

// Checkpoint reconstruction: payload semantics.

#[test]
fn event_after_checkpoint_height_is_rejected() {
    let mut checkpoint = burned_indexer().checkpoint();

    let beyond = checkpoint.context.checkpoint_height + 1;

    let forged = BurnTransaction {
        txid: txid(99),
        block_hash: block_hash(99),
        order: CanonicalOrder {
            height: beyond,
            tx_index: 0,
        },
        ash_value: sat(50),
        records: Vec::new(),
    };

    checkpoint.events.push(OrderedAttestationEvent {
        order: forged.order,
        id: AttestationEventId::Burn(forged.txid),
    });

    checkpoint.burns.insert(forged.txid, forged);

    assert_eq!(
        ReferenceIndexer::try_from(checkpoint),
        Err(Guard::WrongCheckpoint),
    );
}

#[test]
fn zero_clear_omega_checkpoint_is_rejected() {
    let mut checkpoint = burned_indexer().checkpoint();

    let genesis_clear_id = ClearId::Genesis([0_u8; 32]);

    checkpoint.clears.get_mut(&genesis_clear_id).unwrap().omega = Sat::ZERO;

    assert_eq!(ReferenceIndexer::try_from(checkpoint), Err(Guard::Domain),);
}

#[test]
fn zero_clear_y_checkpoint_is_rejected() {
    let mut checkpoint = burned_indexer().checkpoint();

    let genesis_clear_id = ClearId::Genesis([0_u8; 32]);

    checkpoint.clears.get_mut(&genesis_clear_id).unwrap().y = Sat::ZERO;

    assert_eq!(ReferenceIndexer::try_from(checkpoint), Err(Guard::Domain),);
}

#[test]
fn noncontiguous_burn_record_ordinal_checkpoint_is_rejected() {
    let mut checkpoint = burned_indexer().checkpoint();

    let burn_txid = *checkpoint.burns.keys().next().unwrap();

    checkpoint.burns.get_mut(&burn_txid).unwrap().records[0].record_index = 1;

    assert_eq!(
        ReferenceIndexer::try_from(checkpoint),
        Err(Guard::WrongShape),
    );
}

#[test]
fn zero_amount_burn_record_checkpoint_is_rejected() {
    let mut checkpoint = burned_indexer().checkpoint();

    let burn_txid = *checkpoint.burns.keys().next().unwrap();

    checkpoint.burns.get_mut(&burn_txid).unwrap().records[0].amount = Sat::ZERO;

    assert_eq!(ReferenceIndexer::try_from(checkpoint), Err(Guard::Domain),);
}

// Wire-level context identity: a fabricated query with placeholder
// network or genesis identity must fail validation, serialization,
// and deserialization even when its schema and architecture hash are
// correct.

#[test]
fn zero_network_id_query_fails_closed() {
    let mut query = burned_indexer().query(ADDRESS_A).unwrap();

    query.context.network_id = [0_u8; 32];

    assert_eq!(
        validate_query(&query),
        Err(QueryValidationError::ZeroNetworkId),
    );

    assert_eq!(
        serialize_query(&query),
        Err(EncodeError::InvalidQuery(
            QueryValidationError::ZeroNetworkId
        )),
    );

    let bytes = serialize_query_unchecked_for_test(&query).unwrap();

    assert_eq!(deserialize_query(&bytes), Err(DecodeError::ZeroNetworkId));
}

#[test]
fn zero_genesis_id_query_fails_closed() {
    let mut query = burned_indexer().query(ADDRESS_A).unwrap();

    query.context.genesis_id = [0_u8; 32];

    assert_eq!(
        validate_query(&query),
        Err(QueryValidationError::ZeroGenesisId),
    );

    let bytes = serialize_query_unchecked_for_test(&query).unwrap();

    assert_eq!(deserialize_query(&bytes), Err(DecodeError::ZeroGenesisId));
}

// Forged-history ingestion: event type comes from the complete
// transition, so a projection paired with a foreign branch is
// rejected rather than indexed as a genuine event.

#[test]
fn compact_ash_certificate_carrying_burn_is_rejected() {
    let mut world = burned_world();

    world.history.transitions.last_mut().unwrap().branch = BranchKind::CompactAsh;

    assert_eq!(from_forged_history(&world), Err(Guard::WrongShape));
}

#[test]
fn burn_branch_without_burn_projection_is_rejected() {
    let mut world = burned_world();

    world.history.transitions.last_mut().unwrap().burn = None;

    assert_eq!(from_forged_history(&world), Err(Guard::WrongShape));
}

#[test]
fn burn_certificate_carrying_clear_is_rejected() {
    let mut world = burned_world();

    world.history.transitions.last_mut().unwrap().clear = Some(ClearProjection {
        omega: sat(1_000),
        y: sat(1_000),
    });

    assert_eq!(from_forged_history(&world), Err(Guard::WrongShape));
}

#[test]
fn burn_certificate_carrying_residue_is_rejected() {
    let mut world = burned_world();

    world
        .history
        .transitions
        .last_mut()
        .unwrap()
        .distribution_residue = Some(DistributionResidueProjection {
        cycle: 0,
        control_input: 0,
        vault_input: None,
        live_residue: sat(1),
        time_locked_residue: sat(1),
    });

    assert_eq!(from_forged_history(&world), Err(Guard::WrongShape));
}

#[test]
fn cycle_certificate_carrying_clear_is_rejected() {
    let mut world = cleared_world();

    world.history.transitions.last_mut().unwrap().branch = BranchKind::Cycle;

    assert_eq!(from_forged_history(&world), Err(Guard::WrongShape));
}

#[test]
fn clear_branch_without_clear_projection_is_rejected() {
    let mut world = cleared_world();

    world.history.transitions.last_mut().unwrap().clear = None;

    assert_eq!(from_forged_history(&world), Err(Guard::WrongShape));
}

#[test]
fn settlement_certificate_carrying_burn_is_rejected() {
    let mut world = world_with_terminal_residue();

    let settlement = world
        .history
        .transitions
        .iter_mut()
        .find(|certificate| certificate.branch == BranchKind::SettleDistribution)
        .unwrap();

    settlement.burn = Some(BurnProjection {
        ash_output: 0,
        ash_value: sat(1),
        records: Vec::new(),
    });

    assert_eq!(from_forged_history(&world), Err(Guard::WrongShape));
}

#[test]
fn forged_history_burn_record_ordinals_are_rejected() {
    let mut world = burned_world();

    world
        .history
        .transitions
        .last_mut()
        .unwrap()
        .burn
        .as_mut()
        .unwrap()
        .records[0]
        .record_index = 3;

    assert_eq!(from_forged_history(&world), Err(Guard::WrongShape));
}

#[test]
fn forged_history_zero_clear_omega_is_rejected() {
    let mut world = cleared_world();

    world
        .history
        .transitions
        .last_mut()
        .unwrap()
        .clear
        .as_mut()
        .unwrap()
        .omega = Sat::ZERO;

    assert_eq!(from_forged_history(&world), Err(Guard::Domain));
}

// The kernel-derived paths continue to construct: the boundary
// rejects forgeries, not genuine histories or checkpoints.

#[test]
fn genuine_history_and_checkpoint_round_trip() {
    let indexer = burned_indexer();

    let reconstructed = ReferenceIndexer::try_from(indexer.checkpoint()).unwrap();

    assert_eq!(indexer, reconstructed);

    let query = reconstructed.query(ADDRESS_A).unwrap();

    assert_eq!(validate_query(&query), Ok(()));

    let cleared = cleared_world();

    from_forged_history(&cleared).unwrap();
}
