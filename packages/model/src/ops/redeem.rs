//! Redemption.
//!
//! Implements `(´branch:operations:redeem´)`.
//!
//! The generic certificate derivation emits:
//!
//! - STATE `Succ`;
//! - RESV `Succ` for non-sealing redemption;
//! - RESV `Term` for sealing redemption.

use crate::asset::{Asset, ReceiptClass};
use crate::fee::FeeEnvelope;
use crate::guard::Guard;
use crate::history::BranchKind;
use crate::kernel::{OpenFlow, OpenFlowKind, TxBuilder, destruction_flow};
use crate::object::{Meta, Tag};
use crate::pool::PoolState;
use crate::recognition::read_receipt;
use crate::scalar::{CanonicalOrder, OutPoint, Sat, floor_mul_div};
use crate::signer::{SignerSet, require_signer};
use crate::transition::Transition;
use crate::world::World;

// ´branch:operations:redeem´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedeemReceipt {
    pub receipt: OutPoint,
    pub signers: SignerSet,
    pub fee_envelope: FeeEnvelope,
}

impl Transition for RedeemReceipt {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        let (state_outpoint, state) = world.state()?;

        let (resv_outpoint, resv) = world.active_resv()?;

        let receipt = read_receipt(world.utxo(self.receipt)?)?;

        if receipt.class != ReceiptClass::Live {
            return Err(Guard::ClassCross);
        }

        require_signer(&self.signers, receipt.owner)?;

        if receipt.value > state.y_l {
            return Err(Guard::OverDraw);
        }

        let y = state.y()?;

        let sealing = receipt.value == y && state.q.is_zero();

        if receipt.value >= y && !sealing {
            return Err(Guard::NoTrap);
        }

        let payout = floor_mul_div(receipt.value, state.omega, y)?;

        let omega_next = state.omega.checked_sub(payout)?;

        let y_l_next = state.y_l.checked_sub(receipt.value)?;

        let mut tx = TxBuilder::new(world, BranchKind::Redeem, order);

        tx.consume(state_outpoint)?;
        tx.consume(resv_outpoint)?;
        tx.consume(self.receipt)?;

        let payout_output = tx.emit(
            Asset::Lbtc,
            payout,
            Meta::PlainLbtc {
                owner: receipt.owner,
            },
        );

        tx.declare_flow(destruction_flow(
            Asset::U,
            vec![self.receipt],
            Tag::Redeem,
            receipt.value,
        ))?;

        let mut reserve_destinations = vec![payout_output];

        if sealing {
            if payout != state.omega || resv.value != payout {
                return Err(Guard::ResvWeld);
            }

            tx.emit(
                Asset::Pid,
                Sat::ONE,
                Meta::State(PoolState {
                    omega: Sat::ZERO,
                    y_l: Sat::ZERO,
                    y_t: Sat::ZERO,
                    q: Sat::ZERO,
                    cycle: state.cycle,
                    maturity: state.maturity,
                }),
            );
        } else {
            let successor_state = PoolState {
                omega: omega_next,
                y_l: y_l_next,
                ..state
            };

            let successor_reserve = resv.value.checked_sub(payout)?;

            if successor_reserve != successor_state.omega.checked_add(successor_state.q)? {
                return Err(Guard::ResvWeld);
            }

            tx.emit(Asset::Pid, Sat::ONE, Meta::State(successor_state));

            let resv_output = tx.emit(Asset::Lbtc, successor_reserve, Meta::Resv);

            reserve_destinations.push(resv_output);
        }

        // The payout is formula-bound. Sponsor inputs cannot reduce it
        // because sponsor L-BTC belongs to a separate exact flow.
        tx.declare_open_flow(OpenFlow {
            kind: OpenFlowKind::Redemption,
            source_inputs: vec![resv_outpoint],
            destination_outputs: reserve_destinations,
            fee: Sat::ZERO,
        })?;

        tx.apply_fee_envelope(&self.fee_envelope)?;

        Ok(tx.finish()?.world)
    }
}
