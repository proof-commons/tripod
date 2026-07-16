//! Pure monetary query helpers.
//!
//! Implements `(´rule:verification:monetary-queries´)`.
//!
//! These functions consume committed STATE only. They cannot read
//! historical residue.

use crate::guard::Guard;
use crate::scalar::{Sat, floor_mul_div};
use crate::world::World;

// ´rule:verification:monetary-queries´

pub fn floor_terms(world: &World) -> Result<(Sat, Sat), Guard> {
    let state = world.state()?.1;
    let y = state.y()?;

    if y.is_zero() {
        return Err(Guard::Sealed);
    }

    Ok((state.omega, y))
}

pub fn redemption_payout(world: &World, receipt_value: Sat) -> Result<Sat, Guard> {
    let state = world.state()?.1;
    let y = state.y()?;

    if y.is_zero() {
        return Err(Guard::Sealed);
    }

    floor_mul_div(receipt_value, state.omega, y)
}

pub fn cycle_issuance_query(world: &World) -> Result<Sat, Guard> {
    let state = world.state()?.1;
    let y = state.y()?;

    if y.is_zero() {
        return Err(Guard::Sealed);
    }

    if state.q.is_zero() {
        Ok(Sat::ZERO)
    } else {
        floor_mul_div(state.q, y, state.omega)
    }
}
