//! Root cursors, wallets, adversarial environment, and the complete
//! pure `World`.
//!
//! Implements `(´def:verification:root-cursor´)`,
//! `(´def:verification:wallets´)`, `(´def:verification:external-budget´)`,
//! and `(´def:verification:world´)`.

use std::collections::BTreeMap;

use crate::asset::Asset;
use crate::constants::Constants;
use crate::guard::Guard;
use crate::history::History;
use crate::object::{Meta, Utxo};
use crate::pool::PoolState;
use crate::scalar::{OutPoint, OwnerKey, Sat, TxId, checked_active_backing};

// ´def:verification:root-cursor´

/// `resv` is `None` only at the sealed terminal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootCursor {
    pub state: OutPoint,
    pub resv: Option<OutPoint>,
    pub pace: OutPoint,
    pub entitlement_authority: OutPoint,
    pub distribution_authority: OutPoint,
}

// ´def:verification:wallets´

// FUTURE WORK — reserved, not consumed by any v13 operation.
//
// `Wallets` models named-owner balance accounting for a later
// installment (fee-market realism: sponsor balance drawdown across a
// trace, wallet-level conservation checks). In v13 all open-value
// funding flows through `ExternalBudget` plus injected `PLAIN_LBTC`
// outputs, and no operation reads or writes this map. It is retained
// in `World` so its introduction is not a state-shape break later.
// Tracking: v14 candidate; see the realization's future-work note.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Wallets {
    pub balances: BTreeMap<OwnerKey, Sat>,
}

impl Wallets {
    pub fn balance(&self, owner: OwnerKey) -> Sat {
        self.balances.get(&owner).copied().unwrap_or(Sat::ZERO)
    }

    pub fn credit(&mut self, owner: OwnerKey, amount: Sat) -> Result<(), Guard> {
        let next = self.balance(owner).checked_add(amount)?;

        self.balances.insert(owner, next);
        Ok(())
    }

    pub fn debit(&mut self, owner: OwnerKey, amount: Sat) -> Result<(), Guard> {
        let next = self.balance(owner).checked_sub(amount)?;

        self.balances.insert(owner, next);
        Ok(())
    }
}

// ´def:verification:external-budget´

/// The adversary has no external budget for canonical closed assets.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExternalBudget {
    pub lbtc: Sat,
    pub foreign: BTreeMap<u32, Sat>,
}

impl ExternalBudget {
    pub fn debit(&mut self, asset: Asset, amount: Sat) -> Result<(), Guard> {
        match asset {
            Asset::Lbtc => {
                self.lbtc = self.lbtc.checked_sub(amount)?;
            }

            Asset::Foreign(id) => {
                let current = self.foreign.get(&id).copied().unwrap_or(Sat::ZERO);

                self.foreign.insert(id, current.checked_sub(amount)?);
            }

            _ => {
                return Err(Guard::BadAuthorization);
            }
        }

        Ok(())
    }
}

// ´def:verification:world´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    pub utxos: BTreeMap<OutPoint, Utxo>,
    pub roots: RootCursor,

    pub wallets: Wallets,
    pub adversary: ExternalBudget,

    pub history: History,
    pub constants: Constants,

    pub next_outpoint: OutPoint,
    pub next_tx_nonce: u64,

    // Consensus-time abstraction for the current PACE output.
    pub pace_age_blocks: u64,
}

impl World {
    pub fn utxo(&self, outpoint: OutPoint) -> Result<&Utxo, Guard> {
        self.utxos.get(&outpoint).ok_or(Guard::NoSuch)
    }

    pub fn state(&self) -> Result<(OutPoint, PoolState), Guard> {
        let outpoint = self.roots.state;
        let utxo = self.utxo(outpoint)?;

        match (utxo.asset, utxo.meta, utxo.value) {
            (Asset::Pid, Meta::State(state), Sat::ONE) => Ok((outpoint, state)),

            _ => Err(Guard::WrongShape),
        }
    }

    pub fn active_resv(&self) -> Result<(OutPoint, &Utxo), Guard> {
        let outpoint = self.roots.resv.ok_or(Guard::Sealed)?;

        let utxo = self.utxo(outpoint)?;

        match (utxo.asset, utxo.meta) {
            (Asset::Lbtc, Meta::Resv) => Ok((outpoint, utxo)),

            _ => Err(Guard::ResvWeld),
        }
    }

    /// Cap-checked active backing Ω + Q of the current pool state.
    pub fn active_backing(&self) -> Result<Sat, Guard> {
        let state = self.state()?.1;

        checked_active_backing(state.omega, state.q)
    }

    pub fn next_outpoint(&mut self) -> Result<OutPoint, Guard> {
        let outpoint = self.next_outpoint;

        self.next_outpoint = self.next_outpoint.checked_add(1).ok_or(Guard::Overflow)?;

        Ok(outpoint)
    }

    /// The deployment supplies the genesis txid, so it may equal the
    /// id this generator would produce for some future nonce. That
    /// nonce is skipped: without the skip, history replay would
    /// reject the duplicate while the failed transition leaves the
    /// predecessor — and therefore the nonce — unchanged, permanently
    /// wedging an otherwise valid world at its first colliding
    /// transition. At most one nonce can collide, so the loop takes
    /// at most two iterations.
    pub fn next_txid(&mut self) -> Result<TxId, Guard> {
        loop {
            let nonce = self.next_tx_nonce;

            self.next_tx_nonce = self.next_tx_nonce.checked_add(1).ok_or(Guard::Overflow)?;

            let mut bytes = [0_u8; 32];
            bytes[..8].copy_from_slice(&nonce.to_be_bytes());

            let txid = TxId(bytes);

            if txid != self.history.genesis.txid {
                return Ok(txid);
            }
        }
    }
}
