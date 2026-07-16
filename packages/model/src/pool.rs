//! Pool state.
//!
//! Implements `(´def:state:pool-state´)`.

use crate::asset::Maturity;
use crate::guard::Guard;
use crate::scalar::{Cycle, Sat};

// ´def:state:pool-state´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolState {
    pub omega: Sat,
    pub y_l: Sat,
    pub y_t: Sat,
    pub q: Sat,
    pub cycle: Cycle,
    pub maturity: Maturity,
}

impl PoolState {
    pub fn y(self) -> Result<Sat, Guard> {
        self.y_l.checked_add(self.y_t)
    }

    pub fn is_sealed(self) -> Result<bool, Guard> {
        Ok(self.y()?.is_zero() && self.omega.is_zero() && self.q.is_zero())
    }

    pub fn next_cycle(self) -> Result<Cycle, Guard> {
        self.cycle.checked_add(1).ok_or(Guard::CycleOverflow)
    }
}
