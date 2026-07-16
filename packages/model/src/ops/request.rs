//! Request creation and cancellation.
//!
//! Implements `(´protocol:operations:create-request´)` and
//! `(´branch:operations:cancel-request´)`.

use crate::asset::Asset;
use crate::fee::{FeeChange, FeeEnvelope};
use crate::guard::Guard;
use crate::history::BranchKind;
use crate::kernel::{OpenFlow, OpenFlowKind, TxBuilder, ensure_distinct};
use crate::object::Meta;
use crate::scalar::{CanonicalOrder, OutPoint, OwnerKey, Sat};
use crate::signer::{SignerSet, require_signer};
use crate::transition::Transition;
use crate::world::World;

// ´protocol:operations:create-request´

/// The creation profile is client-side policy. Arbitrary malformed
/// request-shaped L-BTC may still exist and remains inert until
/// admission attempts to consume it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateRequest {
    /// Owner of every funding input. This is an enforced semantic
    /// fact, not client-side attribution: an input owned by anyone
    /// else fails the transition even when that owner signed.
    pub funder: OwnerKey,

    pub funding_inputs: Vec<OutPoint>,
    pub signers: SignerSet,

    pub refund_key: OwnerKey,
    pub receipt_owner: OwnerKey,

    pub deposit_principal: Sat,
    pub gross_value: Sat,

    pub change: Option<FeeChange>,
    pub chain_fee: Sat,
}

impl Transition for CreateRequest {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        if self.deposit_principal.is_zero() || self.deposit_principal >= self.gross_value {
            return Err(Guard::PartitionPin);
        }

        if self.funding_inputs.is_empty()
            || self.funding_inputs.len() > world.constants.fee_sponsor_input_max
        {
            return Err(Guard::Domain);
        }

        ensure_distinct(&self.funding_inputs)?;

        let mut tx = TxBuilder::new(world, BranchKind::CreateRequest, order);

        let mut sources = Vec::new();

        for input in &self.funding_inputs {
            let utxo = tx.consume(*input)?;

            let owner = match (utxo.asset, utxo.meta) {
                (Asset::Lbtc, Meta::PlainLbtc { owner }) => owner,

                _ => {
                    return Err(Guard::SponsorMismatch);
                }
            };

            if owner != self.funder {
                return Err(Guard::SponsorMismatch);
            }

            require_signer(&self.signers, owner)?;

            sources.push(*input);
        }

        let request_output = tx.emit(
            Asset::Lbtc,
            self.gross_value,
            Meta::DepositRequest {
                pool_id: world.constants.pool_id,
                refund_key: self.refund_key,
                receipt_owner: self.receipt_owner,
                deposit_principal: self.deposit_principal,
            },
        );

        let mut destinations = vec![request_output];

        if let Some(change) = self.change {
            if change.value.is_zero() {
                return Err(Guard::ZeroProgress);
            }

            destinations.push(tx.emit(
                Asset::Lbtc,
                change.value,
                Meta::PlainLbtc {
                    owner: change.owner,
                },
            ));
        }

        tx.declare_open_flow(OpenFlow {
            kind: OpenFlowKind::RequestCreation,
            source_inputs: sources,
            destination_outputs: destinations,
            fee: self.chain_fee,
        })?;

        tx.set_chain_fee(self.chain_fee);

        Ok(tx.finish()?.world)
    }
}

// ´branch:operations:cancel-request´

/// The complete request value returns to the refund key. The generic
/// fee envelope funds cancellation separately.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CancelRequest {
    pub request: OutPoint,
    pub signers: SignerSet,
    pub fee_envelope: FeeEnvelope,
}

impl Transition for CancelRequest {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        let request = world.utxo(self.request)?;

        let refund_key = match (request.asset, request.meta) {
            (Asset::Lbtc, Meta::DepositRequest { refund_key, .. }) => refund_key,

            _ => return Err(Guard::WrongShape),
        };

        require_signer(&self.signers, refund_key)?;

        let mut tx = TxBuilder::new(world, BranchKind::CancelRequest, order);

        let consumed_request = tx.consume(self.request)?;

        let refund_output = tx.emit(
            Asset::Lbtc,
            consumed_request.value,
            Meta::PlainLbtc { owner: refund_key },
        );

        tx.declare_open_flow(OpenFlow {
            kind: OpenFlowKind::RequestRefund,
            source_inputs: vec![self.request],
            destination_outputs: vec![refund_output],
            fee: Sat::ZERO,
        })?;

        tx.apply_fee_envelope(&self.fee_envelope)?;

        Ok(tx.finish()?.world)
    }
}
