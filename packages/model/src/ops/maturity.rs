//! Maturity announcement.
//!
//! Implements `´branch:operations:announce-maturity´`.
//!
//! The branch is weld-exempt. Its transaction-shape pin forbids every
//! `RESV_SPK` input and every RESV output.

use crate::asset::{Asset, Maturity};
use crate::fee::FeeEnvelope;
use crate::genesis::OPERATOR_KEY;
use crate::guard::Guard;
use crate::history::BranchKind;
use crate::kernel::TxBuilder;
use crate::object::Meta;
use crate::pool::PoolState;
use crate::scalar::{CanonicalOrder, Cycle, Sat};
use crate::signer::{SignerSet, require_signer};
use crate::transition::Transition;
use crate::world::World;

// ´branch:operations:announce-maturity´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnounceMaturity {
    pub maturity_cycle: Cycle,
    pub signers: SignerSet,
    pub fee_envelope: FeeEnvelope,
}

impl Transition for AnnounceMaturity {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        let (state_outpoint, state) = world.state()?;

        if state.is_sealed()? {
            return Err(Guard::Sealed);
        }

        require_signer(&self.signers, OPERATOR_KEY)?;

        if state.maturity != Maturity::Unannounced {
            return Err(Guard::MaturityAlreadyAnnounced);
        }

        let earliest = state
            .cycle
            .checked_add(world.constants.min_maturity_lead)
            .ok_or(Guard::CycleOverflow)?;

        let latest = state
            .cycle
            .checked_add(world.constants.max_maturity_lead)
            .ok_or(Guard::CycleOverflow)?;

        if self.maturity_cycle < earliest {
            return Err(Guard::MaturityLeadTooShort);
        }

        if self.maturity_cycle > latest {
            return Err(Guard::MaturityLeadTooLong);
        }

        let mut tx = TxBuilder::new(world, BranchKind::AnnounceMaturity, order);

        tx.consume(state_outpoint)?;

        tx.apply_fee_envelope(&self.fee_envelope)?;

        tx.emit(
            Asset::Pid,
            Sat::ONE,
            Meta::State(PoolState {
                maturity: Maturity::Announced {
                    cycle: self.maturity_cycle,
                },
                ..state
            }),
        );

        Ok(tx.finish()?.world)
    }
}
