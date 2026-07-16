//! Configuration and finite bounds.
//!
//! Implements `(´def:domains:constants´)`.

use crate::guard::Guard;
use crate::scalar::{Cycle, Ratio};

// ´def:domains:constants´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constants {
    pub pool_id: u32,

    pub zeta: Ratio,
    pub mint_fee: Ratio,

    pub min_maturity_lead: Cycle,
    pub max_maturity_lead: Cycle,

    pub min_cadence_blocks: u64,
    pub max_cadence_blocks: u64,

    pub admission_batch_max: usize,
    pub settlement_batch_max: usize,
    pub relabel_batch_max: usize,
    pub ash_batch_max: usize,

    pub burn_input_max: usize,
    pub burn_change_max: usize,
    pub burn_record_max: usize,

    pub transfer_input_max: usize,
    pub transfer_output_max: usize,

    pub fee_sponsor_input_max: usize,
}

impl Constants {
    pub fn validate(&self) -> Result<(), Guard> {
        if self.min_maturity_lead == 0 || self.min_maturity_lead > self.max_maturity_lead {
            return Err(Guard::BadConstant);
        }

        if self.min_cadence_blocks >= self.max_cadence_blocks {
            return Err(Guard::BadConstant);
        }

        let finite_bounds = [
            self.admission_batch_max,
            self.settlement_batch_max,
            self.relabel_batch_max,
            self.ash_batch_max,
            self.burn_input_max,
            self.burn_change_max,
            self.burn_record_max,
            self.transfer_input_max,
            self.transfer_output_max,
            self.fee_sponsor_input_max,
        ];

        if finite_bounds.iter().any(|value| *value == 0) {
            return Err(Guard::BadConstant);
        }

        Ok(())
    }
}
