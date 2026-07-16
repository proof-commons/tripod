//! State projection classifier, branch-specific request validator, and
//! branch-specific object readers.
//!
//! Implements `(´def:recognition:state-class´)`,
//! `(´rule:recognition:state-projection´)`,
//! `(´def:recognition:request-view´)`,
//! `(´rule:recognition:request-admission´)`,
//! `(´def:recognition:receipt-view´)`,
//! `(´def:recognition:entitlement-view´)`,
//! `(´def:recognition:distribution-control-view´)`, and
//! `(´def:recognition:ash-view´)`.

use crate::asset::{Asset, ReceiptClass};
use crate::constants::Constants;
use crate::guard::Guard;
use crate::object::{Meta, Utxo};
use crate::scalar::{Cycle, OutPoint, OwnerKey, Sat};
use crate::world::World;

// ´def:recognition:state-class´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalObject {
    State,
    Pace,
    EntitlementAuthority,
    DistributionAuthority,
    Receipt {
        owner: OwnerKey,
        class: ReceiptClass,
    },
    DepositEntitlement {
        owner: OwnerKey,
        target_cycle: Cycle,
    },
    DistributionControl,
    DistributionVault {
        cycle: Cycle,
    },
    Ash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateClass {
    Canonical(CanonicalObject),
    ActiveResv,
    InertExternal,
    CanonicalViolation,
}

// ´rule:recognition:state-projection´

pub fn classify_state_object(world: &World, outpoint: OutPoint, utxo: &Utxo) -> StateClass {
    if world.roots.resv == Some(outpoint) {
        return match (utxo.asset, utxo.meta) {
            (Asset::Lbtc, Meta::Resv) => StateClass::ActiveResv,

            _ => StateClass::CanonicalViolation,
        };
    }

    match utxo.asset {
        Asset::Lbtc | Asset::Foreign(_) => StateClass::InertExternal,

        Asset::Pid => match (utxo.meta, utxo.value) {
            (Meta::State(_), Sat::ONE) => StateClass::Canonical(CanonicalObject::State),

            _ => StateClass::CanonicalViolation,
        },

        Asset::Pace => match (utxo.meta, utxo.value) {
            (Meta::Pace, Sat::ONE) => StateClass::Canonical(CanonicalObject::Pace),

            _ => StateClass::CanonicalViolation,
        },

        Asset::EntAuth => match (utxo.meta, utxo.value) {
            (Meta::EntitlementAuthority, Sat::ONE) => {
                StateClass::Canonical(CanonicalObject::EntitlementAuthority)
            }

            _ => StateClass::CanonicalViolation,
        },

        Asset::DistAuth => match (utxo.meta, utxo.value) {
            (Meta::DistributionAuthority, Sat::ONE) => {
                StateClass::Canonical(CanonicalObject::DistributionAuthority)
            }

            _ => StateClass::CanonicalViolation,
        },

        Asset::DistCtl => match (utxo.meta, utxo.value) {
            (Meta::DistributionControl { .. }, Sat::ONE) => {
                StateClass::Canonical(CanonicalObject::DistributionControl)
            }

            _ => StateClass::CanonicalViolation,
        },

        Asset::U => match utxo.meta {
            Meta::Receipt { owner, class } if !utxo.value.is_zero() => {
                StateClass::Canonical(CanonicalObject::Receipt { owner, class })
            }

            Meta::DistributionVault { cycle } if !utxo.value.is_zero() => {
                StateClass::Canonical(CanonicalObject::DistributionVault { cycle })
            }

            Meta::Ash if !utxo.value.is_zero() => StateClass::Canonical(CanonicalObject::Ash),

            _ => StateClass::CanonicalViolation,
        },

        Asset::Ent => match utxo.meta {
            Meta::DepositEntitlement {
                owner,
                target_cycle,
            } if !utxo.value.is_zero() => {
                StateClass::Canonical(CanonicalObject::DepositEntitlement {
                    owner,
                    target_cycle,
                })
            }

            _ => StateClass::CanonicalViolation,
        },
    }
}

// ´def:recognition:request-view´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestView {
    pub refund_key: OwnerKey,
    pub receipt_owner: OwnerKey,
    pub gross_value: Sat,
    pub deposit_principal: Sat,
    pub admission_budget: Sat,
}

// ´rule:recognition:request-admission´

pub fn validate_request_for_admission(
    constants: &Constants,
    utxo: &Utxo,
) -> Result<RequestView, Guard> {
    if utxo.asset != Asset::Lbtc {
        return Err(Guard::WrongAsset);
    }

    let (pool_id, refund_key, receipt_owner, deposit_principal) = match utxo.meta {
        Meta::DepositRequest {
            pool_id,
            refund_key,
            receipt_owner,
            deposit_principal,
        } => (pool_id, refund_key, receipt_owner, deposit_principal),

        _ => return Err(Guard::WrongShape),
    };

    if pool_id != constants.pool_id {
        return Err(Guard::WrongPool);
    }

    if deposit_principal.is_zero() || deposit_principal >= utxo.value {
        return Err(Guard::PartitionPin);
    }

    let admission_budget = utxo.value.checked_sub(deposit_principal)?;

    Ok(RequestView {
        refund_key,
        receipt_owner,
        gross_value: utxo.value,
        deposit_principal,
        admission_budget,
    })
}

// ´def:recognition:receipt-view´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiptView {
    pub owner: OwnerKey,
    pub class: ReceiptClass,
    pub value: Sat,
}

pub fn read_receipt(utxo: &Utxo) -> Result<ReceiptView, Guard> {
    if utxo.asset != Asset::U {
        return Err(Guard::WrongAsset);
    }

    match utxo.meta {
        Meta::Receipt { owner, class } if !utxo.value.is_zero() => Ok(ReceiptView {
            owner,
            class,
            value: utxo.value,
        }),

        _ => Err(Guard::WrongShape),
    }
}

// ´def:recognition:entitlement-view´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntitlementView {
    pub owner: OwnerKey,
    pub target_cycle: Cycle,
    pub value: Sat,
}

pub fn read_entitlement(utxo: &Utxo) -> Result<EntitlementView, Guard> {
    if utxo.asset != Asset::Ent {
        return Err(Guard::WrongAsset);
    }

    match utxo.meta {
        Meta::DepositEntitlement {
            owner,
            target_cycle,
        } if !utxo.value.is_zero() => Ok(EntitlementView {
            owner,
            target_cycle,
            value: utxo.value,
        }),

        _ => Err(Guard::WrongShape),
    }
}

// ´def:recognition:distribution-control-view´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DistributionControlView {
    pub cycle: Cycle,
    pub principal: Sat,
    pub live_allocation: Sat,
    pub time_locked_allocation: Sat,
    pub remaining_principal: Sat,
    pub remaining_live_value: Sat,
    pub remaining_time_locked_value: Sat,
}

pub fn read_distribution_control(utxo: &Utxo) -> Result<DistributionControlView, Guard> {
    if utxo.asset != Asset::DistCtl || utxo.value != Sat::ONE {
        return Err(Guard::WrongAsset);
    }

    match utxo.meta {
        Meta::DistributionControl {
            cycle,
            principal,
            live_allocation,
            time_locked_allocation,
            remaining_principal,
            remaining_live_value,
            remaining_time_locked_value,
        } => {
            if principal.is_zero()
                || remaining_principal.is_zero()
                || remaining_principal > principal
                || remaining_live_value > live_allocation
                || remaining_time_locked_value > time_locked_allocation
            {
                return Err(Guard::Domain);
            }

            Ok(DistributionControlView {
                cycle,
                principal,
                live_allocation,
                time_locked_allocation,
                remaining_principal,
                remaining_live_value,
                remaining_time_locked_value,
            })
        }

        _ => Err(Guard::WrongShape),
    }
}

// ´def:recognition:ash-view´

pub fn read_ash(utxo: &Utxo) -> Result<Sat, Guard> {
    if utxo.asset != Asset::U {
        return Err(Guard::WrongAsset);
    }

    match utxo.meta {
        Meta::Ash if !utxo.value.is_zero() => Ok(utxo.value),

        _ => Err(Guard::WrongShape),
    }
}
