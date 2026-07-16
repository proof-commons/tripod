//! Receipt relabel.
//!
//! Implements `´def:operations:relabel-receipts´` and
//! `(´branch:operations:receipt-relabel´)`.
//!
//! Relabeling uses a canonical positional bijection by transaction
//! layout, not a separate witness that could conflict with
//! sponsor-change output positions:
//!
//! - output 0: STATE successor;
//! - outputs 1..N: corresponding live receipts, in input order;
//! - remaining outputs: optional sponsor change.
//!
//! Each receipt flow names its exact generated output reference. The
//! compiler pins the same layout.

use crate::asset::{Asset, Maturity, ReceiptClass};
use crate::fee::FeeEnvelope;
use crate::guard::Guard;
use crate::history::{BranchKind, DeltaKind};
use crate::kernel::{TxBuilder, ensure_distinct, movement_flow};
use crate::object::Meta;
use crate::recognition::read_receipt;
use crate::scalar::{CanonicalOrder, OutPoint, Sat};
use crate::transition::Transition;
use crate::world::World;

// ´branch:operations:receipt-relabel´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelabelReceipts {
    pub receipts: Vec<OutPoint>,
    pub fee_envelope: FeeEnvelope,
}

impl Transition for RelabelReceipts {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        if self.receipts.is_empty() || self.receipts.len() > world.constants.relabel_batch_max {
            return Err(Guard::Domain);
        }

        ensure_distinct(&self.receipts)?;

        let (state_outpoint, state) = world.state()?;

        if state.maturity != Maturity::Complete {
            return Err(Guard::MaturityNotComplete);
        }

        let mut tx = TxBuilder::new(world, BranchKind::ReceiptRelabel, order);

        tx.consume(state_outpoint)?;

        // Output 0 is always the byte-identical STATE successor.
        tx.emit(Asset::Pid, Sat::ONE, Meta::State(state));

        // Outputs 1..N correspond to inputs 0..N−1.
        for receipt_outpoint in &self.receipts {
            let receipt = read_receipt(world.utxo(*receipt_outpoint)?)?;

            if receipt.class != ReceiptClass::TimeLocked {
                return Err(Guard::ClassCross);
            }

            tx.consume(*receipt_outpoint)?;

            let live_output = tx.emit(
                Asset::U,
                receipt.value,
                Meta::Receipt {
                    owner: receipt.owner,
                    class: ReceiptClass::Live,
                },
            );

            tx.declare_flow(movement_flow(
                Asset::U,
                DeltaKind::Lateral,
                vec![*receipt_outpoint],
                vec![live_output],
            ))?;
        }

        // Sponsor change is emitted only after all protocol outputs.
        tx.apply_fee_envelope(&self.fee_envelope)?;

        Ok(tx.finish()?.world)
    }
}
