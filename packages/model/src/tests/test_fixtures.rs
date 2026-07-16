//! Test fixtures.
//!
//! Implements `(´test:verification:fixtures´)`.

use crate::*;

pub const ALICE: OwnerKey = OwnerKey([1_u8; 32]);

pub const BOB: OwnerKey = OwnerKey([2_u8; 32]);

pub const CAROL: OwnerKey = OwnerKey([3_u8; 32]);

pub const RELAYER: OwnerKey = OwnerKey([4_u8; 32]);

pub const ATTACKER: OwnerKey = OwnerKey([5_u8; 32]);

pub const ADDRESS_A: AttestationAddress = AttestationAddress([11_u8; 32]);

pub fn sat(value: u64) -> Sat {
    Sat::new(value).unwrap()
}

pub fn order(tx_index: u32) -> CanonicalOrder {
    CanonicalOrder {
        height: 1,
        tx_index,
    }
}

pub fn later_order(height: u64, tx_index: u32) -> CanonicalOrder {
    CanonicalOrder { height, tx_index }
}

pub fn txid(nonce: u8) -> TxId {
    let mut bytes = [0_u8; 32];

    bytes[31] = nonce;

    TxId(bytes)
}

pub fn block_hash(nonce: u8) -> BlockHash {
    let mut bytes = [0_u8; 32];

    bytes[31] = nonce;

    BlockHash(bytes)
}

pub const TEST_NETWORK_ID: [u8; 32] = [1_u8; 32];
pub const TEST_GENESIS_ID: [u8; 32] = [2_u8; 32];

/// The context manifest hash is the real typed-architecture semantic
/// hash: the canonical serializer rejects any other value, so fixtures
/// must not use a placeholder.
pub fn test_manifest_hash() -> [u8; 32] {
    expected_architecture_manifest_hash().unwrap()
}

pub fn context(
    checkpoint_height: BlockHeight,
    checkpoint_block_hash: BlockHash,
) -> AttestationContext {
    AttestationContext {
        network_id: TEST_NETWORK_ID,
        genesis_id: TEST_GENESIS_ID,
        architecture_manifest_hash: test_manifest_hash(),
        checkpoint_block_hash,
        checkpoint_height,
        schema_version: ATTESTATION_SCHEMA_VERSION,
    }
}

/// Builds a contiguous synthetic chain view spanning the world's
/// history, checkpointed at the last transition height. For histories
/// whose transitions share one height, this produces one block.
pub fn chain_view_for_history(world: &World) -> ValidatedChainView {
    let genesis_height = world.history.genesis.order.height;

    let checkpoint_height = world
        .history
        .transitions
        .last()
        .map(|transition| transition.order.height)
        .unwrap_or(genesis_height);

    let mut blocks = Vec::new();
    let mut previous_hash = None;

    for height in genesis_height..=checkpoint_height {
        let hash = block_hash(u8::try_from(height % 256).unwrap());

        blocks.push(CanonicalBlock {
            height,
            hash,
            parent_hash: previous_hash,
        });

        previous_hash = Some(hash);
    }

    let checkpoint_hash = blocks.last().unwrap().hash;

    ValidatedChainView::new(
        TEST_NETWORK_ID,
        TEST_GENESIS_ID,
        test_manifest_hash(),
        checkpoint_height,
        checkpoint_hash,
        ATTESTATION_SCHEMA_VERSION,
        blocks,
    )
    .unwrap()
}

pub fn constants() -> Constants {
    Constants {
        pool_id: 7,

        zeta: Ratio::new(1, 2).unwrap(),

        mint_fee: Ratio::new(1, 2).unwrap(),

        min_maturity_lead: 10,

        max_maturity_lead: 1000,

        min_cadence_blocks: 10,

        max_cadence_blocks: 100,

        admission_batch_max: 32,

        settlement_batch_max: 32,

        relabel_batch_max: 32,

        ash_batch_max: 64,

        burn_input_max: 64,

        burn_change_max: 32,

        burn_record_max: 64,

        transfer_input_max: 64,

        transfer_output_max: 64,

        fee_sponsor_input_max: 16,
    }
}

pub fn world() -> World {
    genesis(
        constants(),
        sat(1_000_000),
        CanonicalOrder {
            height: 0,
            tx_index: 0,
        },
        txid(0),
    )
    .unwrap()
}

pub fn find_live_receipt(world: &World, owner: OwnerKey) -> OutPoint {
    world
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (
                Asset::U,
                Meta::Receipt {
                    owner: receipt_owner,
                    class: ReceiptClass::Live,
                },
            ) if receipt_owner == owner => Some(*outpoint),

            _ => None,
        })
        .expect("live receipt")
}

pub fn find_request(world: &World) -> OutPoint {
    world
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| {
            if matches!(
                (utxo.asset, utxo.meta),
                (Asset::Lbtc, Meta::DepositRequest { .. }),
            ) {
                Some(*outpoint)
            } else {
                None
            }
        })
        .expect("request")
}

pub fn empty_fee_envelope() -> FeeEnvelope {
    FeeEnvelope::default()
}

pub fn external_lbtc_world(amount: u64) -> World {
    let mut world = world();

    let available = Sat::new(ACTIVE_BACKING_MAX)
        .unwrap()
        .checked_sub(world.active_resv().unwrap().1.value)
        .unwrap();

    let requested = sat(amount);

    assert!(requested <= available);

    world.adversary.lbtc = requested;

    world
}

pub fn inject_and_get_outpoint(
    world: &World,
    asset: Asset,
    value: Sat,
    meta: Meta,
) -> (World, OutPoint) {
    let outpoint = world.next_outpoint;

    let next = inject_open_object(world, asset, value, meta).unwrap();

    (next, outpoint)
}
