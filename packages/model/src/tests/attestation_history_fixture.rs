//! Synthetic attestation-history fixture.
//!
//! Implements `´test:verification:attestation-history-fixture´`.

use super::scenario_fixtures::*;
use super::test_fixtures::{block_hash, txid};
use crate::*;

pub fn clear_entry(id: u8, height: u64, tx_index: u32, omega: u64, y: u64) -> ClearEntry {
    ClearEntry {
        clear_id: ClearId::Transaction(txid(id)),

        block_hash: block_hash(height as u8),

        order: CanonicalOrder { height, tx_index },

        omega: sat(omega),

        y: sat(y),
    }
}

pub fn accepted_burn(
    id: u8,
    height: u64,
    tx_index: u32,
    amount: u64,
    address: AttestationAddress,
) -> BurnTransaction {
    BurnTransaction {
        txid: txid(id),

        block_hash: block_hash(height as u8),

        order: CanonicalOrder { height, tx_index },

        ash_value: sat(amount),

        records: vec![BurnRecord {
            record_index: 0,
            address,
            amount: sat(amount),
        }],
    }
}

pub fn genesis_clear() -> ClearEntry {
    ClearEntry {
        clear_id: ClearId::Genesis([0_u8; 32]),

        block_hash: block_hash(0),

        order: CanonicalOrder {
            height: 0,
            tx_index: 0,
        },

        omega: sat(1_000),

        y: sat(1_000),
    }
}
