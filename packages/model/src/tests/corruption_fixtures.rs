//! Invariant-corruption helpers.
//!
//! Implements `´test:verification:corruption-fixtures´`.
//!
//! These helpers create invalid low-level states and are used only to
//! test the checker.

use crate::*;

pub fn replace_utxo(world: &World, outpoint: OutPoint, replacement: Utxo) -> World {
    let mut corrupted = world.clone();

    corrupted.utxos.insert(outpoint, replacement);

    corrupted
}

pub fn add_closed_utxo(world: &World, utxo: Utxo) -> World {
    let mut corrupted = world.clone();

    let outpoint = corrupted.next_outpoint;

    corrupted.next_outpoint += 1;

    corrupted.utxos.insert(outpoint, utxo);

    corrupted
}
