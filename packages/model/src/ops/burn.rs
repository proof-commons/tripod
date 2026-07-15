//! Burn.
//!
//! Implements `´branch:operations:burn´`.
//!
//! The kernel derives a `BurnProjection` only if:
//!
//! - branch is `Burn`;
//! - every canonical `U` input is a live receipt;
//! - no ASH input;
//! - exactly one ASH output;
//! - all other canonical `U` outputs are live receipt change;
//! - all output-integrity authorization conditions are met by the
//!   compiler/deployment profile.

use crate::asset::{Asset, ReceiptClass};
use crate::fee::FeeEnvelope;
use crate::guard::Guard;
use crate::history::{BranchKind, BurnRecord, DeltaKind};
use crate::kernel::{TxBuilder, ensure_distinct, movement_flow};
use crate::object::Meta;
use crate::ops::transfer::ReceiptDestination;
use crate::recognition::read_receipt;
use crate::scalar::{CanonicalOrder, OutPoint, Sat};
use crate::signer::{SignerSet, require_signer};
use crate::transition::Transition;
use crate::world::World;

// ´branch:operations:burn´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurnReceipts {
    pub receipts: Vec<OutPoint>,
    pub signers: SignerSet,

    pub ash_value: Sat,

    pub change: Vec<ReceiptDestination>,
    pub records: Vec<BurnRecord>,

    pub fee_envelope: FeeEnvelope,
}

impl Transition for BurnReceipts {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        if self.receipts.is_empty()
            || self.receipts.len() > world.constants.burn_input_max
            || self.change.len() > world.constants.burn_change_max
            || self.records.len() > world.constants.burn_record_max
            || self.ash_value.is_zero()
        {
            return Err(Guard::Domain);
        }

        ensure_distinct(&self.receipts)?;

        let mut input_total = Sat::ZERO;

        for input in &self.receipts {
            let receipt = read_receipt(world.utxo(*input)?)?;

            if receipt.class != ReceiptClass::Live {
                return Err(Guard::ClassCross);
            }

            require_signer(&self.signers, receipt.owner)?;

            input_total = input_total.checked_add(receipt.value)?;
        }

        let mut change_total = Sat::ZERO;

        for change in &self.change {
            if change.value.is_zero() {
                return Err(Guard::ZeroProgress);
            }

            change_total = change_total.checked_add(change.value)?;
        }

        if self.ash_value.checked_add(change_total)? != input_total {
            return Err(Guard::ValuePin);
        }

        let mut tx = TxBuilder::new(world, BranchKind::Burn, order);

        for input in &self.receipts {
            tx.consume(*input)?;
        }

        tx.apply_fee_envelope(&self.fee_envelope)?;

        let ash_output = tx.emit(Asset::U, self.ash_value, Meta::Ash);

        let mut movement_destinations = vec![ash_output];

        for change in &self.change {
            movement_destinations.push(tx.emit(
                Asset::U,
                change.value,
                Meta::Receipt {
                    owner: change.owner,
                    class: ReceiptClass::Live,
                },
            ));
        }

        tx.declare_flow(movement_flow(
            Asset::U,
            DeltaKind::Lateral,
            self.receipts.clone(),
            movement_destinations,
        ))?;

        for record in &self.records {
            tx.emit_burn_record(*record)?;
        }

        Ok(tx.finish()?.world)
    }
}
