//! External open-asset injection.
//!
//! Implements `(´def:verification:open-object-injection´)`.
//!
//! Open-asset junk may be request-shaped, RESV-shaped, anchor-shaped,
//! or state-shaped. It remains inert unless a branch-specific validator
//! accepts it.

use crate::asset::Asset;
use crate::guard::Guard;
use crate::invariant::check_invariant;
use crate::object::{Meta, Utxo};
use crate::scalar::Sat;
use crate::world::World;

// ´def:verification:open-object-injection´

pub fn inject_open_object(
    world: &World,
    asset: Asset,
    value: Sat,
    meta: Meta,
) -> Result<World, Guard> {
    if !asset.is_open() {
        return Err(Guard::BadAuthorization);
    }

    let mut next = world.clone();

    next.adversary.debit(asset, value)?;

    let outpoint = next.next_outpoint()?;

    next.utxos.insert(outpoint, Utxo { asset, value, meta });

    check_invariant(&next).map_err(|_| Guard::InvariantFailure)?;

    Ok(next)
}
