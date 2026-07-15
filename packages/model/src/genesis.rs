//! Genesis constructor.
//!
//! Implements `´protocol:state:genesis´`.
//!
//! Genesis is trusted setup, recorded as the dedicated
//! `´def:verification:genesis-history´` projection carried by
//! [`crate::history::History`], never as an ordinary branch kind — but
//! its resulting state is still checked against the full invariant.

use std::collections::BTreeMap;

use crate::asset::{Asset, Maturity, ReceiptClass};
use crate::constants::Constants;
use crate::guard::Guard;
use crate::history::{GenesisProjection, History};
use crate::invariant::check_invariant;
use crate::manifest::validate_bound_conformance;
use crate::object::{Meta, Utxo};
use crate::pool::PoolState;
use crate::scalar::{
    CanonicalOrder, OutPoint, OwnerKey, Sat, TxId, floor_ratio, require_active_backing_cap,
};
use crate::world::{ExternalBudget, RootCursor, Wallets, World};

// ´protocol:state:genesis´

pub const GENESIS_OWNER: OwnerKey = OwnerKey([0_u8; 32]);

pub const OPERATOR_KEY: OwnerKey = GENESIS_OWNER;

pub fn genesis(
    constants: Constants,
    genesis_reserve: Sat,
    genesis_order: CanonicalOrder,
    genesis_txid: TxId,
) -> Result<World, Guard> {
    constants.validate()?;

    // Raw model-world construction and deployment-profile validation
    // share the same bound authority: a world whose finite bounds
    // cannot satisfy the architecture's cardinality minima must not
    // exist, not merely fail profile validation later.
    validate_bound_conformance(&constants)?;

    if genesis_reserve.is_zero() {
        return Err(Guard::Domain);
    }

    require_active_backing_cap(genesis_reserve)?;

    let live = floor_ratio(genesis_reserve, constants.zeta)?;

    if live.is_zero() || live == genesis_reserve {
        return Err(Guard::BadConstant);
    }

    let time_locked = genesis_reserve.checked_sub(live)?;

    let mut utxos: BTreeMap<OutPoint, Utxo> = BTreeMap::new();
    let mut next_outpoint: OutPoint = 0;

    let allocate = |utxos: &mut BTreeMap<OutPoint, Utxo>,
                    next_outpoint: &mut OutPoint,
                    utxo: Utxo|
     -> Result<OutPoint, Guard> {
        let outpoint = *next_outpoint;

        *next_outpoint = next_outpoint.checked_add(1).ok_or(Guard::Overflow)?;

        utxos.insert(outpoint, utxo);
        Ok(outpoint)
    };

    let state_outpoint = allocate(
        &mut utxos,
        &mut next_outpoint,
        Utxo {
            asset: Asset::Pid,
            value: Sat::ONE,
            meta: Meta::State(PoolState {
                omega: genesis_reserve,
                y_l: live,
                y_t: time_locked,
                q: Sat::ZERO,
                cycle: 0,
                maturity: Maturity::Unannounced,
            }),
        },
    )?;

    let resv_outpoint = allocate(
        &mut utxos,
        &mut next_outpoint,
        Utxo {
            asset: Asset::Lbtc,
            value: genesis_reserve,
            meta: Meta::Resv,
        },
    )?;

    let pace_outpoint = allocate(
        &mut utxos,
        &mut next_outpoint,
        Utxo {
            asset: Asset::Pace,
            value: Sat::ONE,
            meta: Meta::Pace,
        },
    )?;

    let entitlement_authority_outpoint = allocate(
        &mut utxos,
        &mut next_outpoint,
        Utxo {
            asset: Asset::EntAuth,
            value: Sat::ONE,
            meta: Meta::EntitlementAuthority,
        },
    )?;

    let distribution_authority_outpoint = allocate(
        &mut utxos,
        &mut next_outpoint,
        Utxo {
            asset: Asset::DistAuth,
            value: Sat::ONE,
            meta: Meta::DistributionAuthority,
        },
    )?;

    let live_receipt_outpoint = allocate(
        &mut utxos,
        &mut next_outpoint,
        Utxo {
            asset: Asset::U,
            value: live,
            meta: Meta::Receipt {
                owner: GENESIS_OWNER,
                class: ReceiptClass::Live,
            },
        },
    )?;

    let time_locked_receipt_outpoint = allocate(
        &mut utxos,
        &mut next_outpoint,
        Utxo {
            asset: Asset::U,
            value: time_locked,
            meta: Meta::Receipt {
                owner: GENESIS_OWNER,
                class: ReceiptClass::TimeLocked,
            },
        },
    )?;

    // Trusted-setup txid supplied by the deployment.
    let genesis_projection = GenesisProjection {
        txid: genesis_txid,
        order: genesis_order,

        state_out: state_outpoint,
        resv_out: resv_outpoint,
        pace_out: pace_outpoint,
        entitlement_authority_out: entitlement_authority_outpoint,
        distribution_authority_out: distribution_authority_outpoint,

        live_receipt_out: live_receipt_outpoint,
        time_locked_receipt_out: time_locked_receipt_outpoint,

        omega: genesis_reserve,
        y: genesis_reserve,
    };

    let world = World {
        utxos,

        roots: RootCursor {
            state: state_outpoint,
            resv: Some(resv_outpoint),
            pace: pace_outpoint,
            entitlement_authority: entitlement_authority_outpoint,
            distribution_authority: distribution_authority_outpoint,
        },

        wallets: Wallets::default(),
        adversary: ExternalBudget::default(),

        history: History {
            genesis: genesis_projection,
            transitions: Vec::new(),
        },
        constants,

        next_outpoint,
        next_tx_nonce: 1,

        pace_age_blocks: 0,
    };

    check_invariant(&world).map_err(|_| Guard::InvariantFailure)?;

    Ok(world)
}
