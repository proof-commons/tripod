//! Receipt transfer.
//!
//! Implements `´protocol:operations:transfer´`.
//!
//! Every required owner authorizes the complete modeled output set.
//! Exact sighash bytes remain a compiler/deployment concern.

use crate::asset::{Asset, ReceiptClass};
use crate::fee::FeeEnvelope;
use crate::guard::Guard;
use crate::history::{BranchKind, DeltaKind};
use crate::kernel::{TxBuilder, ensure_distinct, movement_flow};
use crate::object::Meta;
use crate::recognition::read_receipt;
use crate::scalar::{CanonicalOrder, OutPoint, OwnerKey, Sat};
use crate::signer::{SignerSet, require_signer};
use crate::transition::Transition;
use crate::world::World;

// ´protocol:operations:transfer´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferReceipts {
    pub class: ReceiptClass,
    pub inputs: Vec<OutPoint>,
    pub outputs: Vec<ReceiptDestination>,
    pub signers: SignerSet,
    pub fee_envelope: FeeEnvelope,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiptDestination {
    pub owner: OwnerKey,
    pub value: Sat,
}

impl Transition for TransferReceipts {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        if self.inputs.is_empty()
            || self.inputs.len() > world.constants.transfer_input_max
            || self.outputs.is_empty()
            || self.outputs.len() > world.constants.transfer_output_max
        {
            return Err(Guard::Domain);
        }

        ensure_distinct(&self.inputs)?;

        let mut input_total = Sat::ZERO;

        for input in &self.inputs {
            let receipt = read_receipt(world.utxo(*input)?)?;

            if receipt.class != self.class {
                return Err(Guard::ClassCross);
            }

            require_signer(&self.signers, receipt.owner)?;

            input_total = input_total.checked_add(receipt.value)?;
        }

        let mut output_total = Sat::ZERO;

        for output in &self.outputs {
            if output.value.is_zero() {
                return Err(Guard::ZeroProgress);
            }

            output_total = output_total.checked_add(output.value)?;
        }

        if input_total != output_total {
            return Err(Guard::ValuePin);
        }

        let branch = match self.class {
            ReceiptClass::Live => BranchKind::TransferLive,

            ReceiptClass::TimeLocked => BranchKind::TransferTimeLocked,
        };

        let mut tx = TxBuilder::new(world, branch, order);

        for input in &self.inputs {
            tx.consume(*input)?;
        }

        tx.apply_fee_envelope(&self.fee_envelope)?;

        let mut destination_refs = Vec::new();

        for output in &self.outputs {
            destination_refs.push(tx.emit(
                Asset::U,
                output.value,
                Meta::Receipt {
                    owner: output.owner,
                    class: self.class,
                },
            ));
        }

        tx.declare_flow(movement_flow(
            Asset::U,
            DeltaKind::Lateral,
            self.inputs.clone(),
            destination_refs,
        ))?;

        Ok(tx.finish()?.world)
    }
}
