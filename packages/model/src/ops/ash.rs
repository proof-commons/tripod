//! ASH compaction and clear.
//!
//! Implements `(´branch:operations:compact-ash´)` and
//! `(´branch:operations:clear´)`.
//!
//! Compaction is attestation-silent because its event-type anchor
//! includes ASH inputs. The kernel derives a `ClearProjection` from the
//! predecessor/successor STATE and the authenticated clear branch;
//! clear emits no `BurnProjection`.

use crate::asset::Asset;
use crate::fee::FeeEnvelope;
use crate::guard::Guard;
use crate::history::{BranchKind, DeltaKind};
use crate::kernel::{CanonicalFlow, DestructionLeg, TxBuilder, ensure_distinct, movement_flow};
use crate::object::{Meta, Tag};
use crate::pool::PoolState;
use crate::recognition::read_ash;
use crate::scalar::{CanonicalOrder, OutPoint, Sat};
use crate::transition::Transition;
use crate::world::World;

// ´branch:operations:compact-ash´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactAsh {
    pub ash_inputs: Vec<OutPoint>,
    pub fee_envelope: FeeEnvelope,
}

impl Transition for CompactAsh {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        if self.ash_inputs.len() < 2 || self.ash_inputs.len() > world.constants.ash_batch_max {
            return Err(Guard::Domain);
        }

        ensure_distinct(&self.ash_inputs)?;

        let mut total = Sat::ZERO;

        for input in &self.ash_inputs {
            total = total.checked_add(read_ash(world.utxo(*input)?)?)?;
        }

        let mut tx = TxBuilder::new(world, BranchKind::CompactAsh, order);

        for input in &self.ash_inputs {
            tx.consume(*input)?;
        }

        tx.apply_fee_envelope(&self.fee_envelope)?;

        let output = tx.emit(Asset::U, total, Meta::Ash);

        tx.declare_flow(movement_flow(
            Asset::U,
            DeltaKind::OwnerlessLateral,
            self.ash_inputs.clone(),
            vec![output],
        ))?;

        Ok(tx.finish()?.world)
    }
}

// ´branch:operations:clear´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClearAsh {
    pub ash_inputs: Vec<OutPoint>,
    pub fee_envelope: FeeEnvelope,
}

impl Transition for ClearAsh {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        if self.ash_inputs.is_empty() || self.ash_inputs.len() > world.constants.ash_batch_max {
            return Err(Guard::Domain);
        }

        ensure_distinct(&self.ash_inputs)?;

        let (state_outpoint, state) = world.state()?;

        let y = state.y()?;

        if y.is_zero() {
            return Err(Guard::Sealed);
        }

        let mut ash_total = Sat::ZERO;

        for input in &self.ash_inputs {
            ash_total = ash_total.checked_add(read_ash(world.utxo(*input)?)?)?;
        }

        let y_minus_one = y.checked_sub(Sat::ONE)?;

        // The clamp is why Y = 0 is unreachable from clear — by
        // construction, not rejection. The `y - 1` term is a safety
        // property, not an arithmetic incident: even a batch clearing
        // every live receipt leaves one unit of supply, and the
        // uncleared remainder is re-emitted as ASH rather than
        // stranded.
        let clear_amount = ash_total.min(state.y_l).min(y_minus_one);

        if clear_amount.is_zero() {
            return Err(Guard::ZeroProgress);
        }

        let residual = ash_total.checked_sub(clear_amount)?;

        let state_next = PoolState {
            y_l: state.y_l.checked_sub(clear_amount)?,
            ..state
        };

        let mut tx = TxBuilder::new(world, BranchKind::Clear, order);

        tx.consume(state_outpoint)?;

        for input in &self.ash_inputs {
            tx.consume(*input)?;
        }

        tx.apply_fee_envelope(&self.fee_envelope)?;

        tx.emit(Asset::Pid, Sat::ONE, Meta::State(state_next));

        let mut destination_outputs = Vec::new();

        let movement_kind = if residual.is_zero() {
            None
        } else {
            destination_outputs.push(tx.emit(Asset::U, residual, Meta::Ash));
            Some(DeltaKind::OwnerlessLateral)
        };

        tx.declare_flow(CanonicalFlow {
            asset: Asset::U,
            source_inputs: self.ash_inputs.clone(),
            destination_outputs,
            destructions: vec![DestructionLeg {
                tag: Tag::Recon,
                amount: clear_amount,
            }],
            movement_kind,
        })?;

        Ok(tx.finish()?.world)
    }
}
