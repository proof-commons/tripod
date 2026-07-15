//! Distribution settlement.
//!
//! Implements `´branch:operations:settle-distribution´`.
//!
//! Settlement always emits the distribution's original receipt classes.
//! It never reads current maturity state.

use std::collections::BTreeMap;

use crate::asset::{Asset, ReceiptClass};
use crate::fee::FeeEnvelope;
use crate::guard::Guard;
use crate::history::{BranchKind, DeltaKind};
use crate::kernel::{
    CanonicalFlow, DestructionLeg, TxBuilder, destruction_flow, ensure_distinct, movement_flow,
};
use crate::object::{Meta, Tag};
use crate::recognition::{read_distribution_control, read_entitlement};
use crate::scalar::{CanonicalOrder, OutPoint, OwnerKey, Sat, checked_add_to_map, floor_mul_div};
use crate::transition::Transition;
use crate::world::World;

// ´branch:operations:settle-distribution´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettleDistribution {
    pub control: OutPoint,
    pub vault: Option<OutPoint>,
    pub entitlements: Vec<OutPoint>,
    pub fee_envelope: FeeEnvelope,
}

impl Transition for SettleDistribution {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        if self.entitlements.is_empty()
            || self.entitlements.len() > world.constants.settlement_batch_max
        {
            return Err(Guard::Domain);
        }

        ensure_distinct(&self.entitlements)?;

        let control_utxo = world.utxo(self.control)?;

        let control = read_distribution_control(control_utxo)?;

        let expected_vault_value = control
            .remaining_live_value
            .checked_add(control.remaining_time_locked_value)?;

        let actual_vault_value = match (self.vault, expected_vault_value) {
            (Some(vault_outpoint), value) if !value.is_zero() => {
                let vault = world.utxo(vault_outpoint)?;

                match (vault.asset, vault.meta) {
                    (Asset::U, Meta::DistributionVault { cycle })
                        if cycle == control.cycle && vault.value == value =>
                    {
                        vault.value
                    }

                    _ => {
                        return Err(Guard::ControlVaultWeld);
                    }
                }
            }

            (None, value) if value.is_zero() => Sat::ZERO,

            _ => {
                return Err(Guard::ControlVaultWeld);
            }
        };

        let mut entitlement_views = Vec::new();

        let mut settled_principal = Sat::ZERO;

        let mut live_total = Sat::ZERO;
        let mut time_locked_total = Sat::ZERO;

        let mut receipt_outputs: BTreeMap<(OwnerKey, ReceiptClass), Sat> = BTreeMap::new();

        for entitlement_outpoint in &self.entitlements {
            let entitlement_utxo = world.utxo(*entitlement_outpoint)?;

            let entitlement = read_entitlement(entitlement_utxo)?;

            if entitlement.target_cycle != control.cycle {
                return Err(Guard::WrongTarget);
            }

            let live = floor_mul_div(
                entitlement.value,
                control.live_allocation,
                control.principal,
            )?;

            let time_locked = floor_mul_div(
                entitlement.value,
                control.time_locked_allocation,
                control.principal,
            )?;

            settled_principal = settled_principal.checked_add(entitlement.value)?;

            live_total = live_total.checked_add(live)?;

            time_locked_total = time_locked_total.checked_add(time_locked)?;

            if !live.is_zero() {
                checked_add_to_map(
                    &mut receipt_outputs,
                    (entitlement.owner, ReceiptClass::Live),
                    live,
                )?;
            }

            if !time_locked.is_zero() {
                checked_add_to_map(
                    &mut receipt_outputs,
                    (entitlement.owner, ReceiptClass::TimeLocked),
                    time_locked,
                )?;
            }

            entitlement_views.push((*entitlement_outpoint, entitlement));
        }

        if settled_principal > control.remaining_principal {
            return Err(Guard::OverDraw);
        }

        let remaining_principal = control.remaining_principal.checked_sub(settled_principal)?;

        let remaining_live = control.remaining_live_value.checked_sub(live_total)?;

        let remaining_time_locked = control
            .remaining_time_locked_value
            .checked_sub(time_locked_total)?;

        let moved = live_total.checked_add(time_locked_total)?;

        let successor_vault_value = actual_vault_value.checked_sub(moved)?;

        if successor_vault_value != remaining_live.checked_add(remaining_time_locked)? {
            return Err(Guard::ControlVaultWeld);
        }

        let mut tx = TxBuilder::new(world, BranchKind::SettleDistribution, order);

        tx.consume(self.control)?;

        if let Some(vault) = self.vault {
            tx.consume(vault)?;
        }

        for (entitlement_outpoint, _) in &entitlement_views {
            tx.consume(*entitlement_outpoint)?;
        }

        tx.apply_fee_envelope(&self.fee_envelope)?;

        let mut receipt_output_refs = Vec::new();

        for ((owner, class), value) in receipt_outputs {
            let output = tx.emit(Asset::U, value, Meta::Receipt { owner, class });

            receipt_output_refs.push(output);
        }

        let entitlement_inputs = entitlement_views
            .iter()
            .map(|(outpoint, _)| *outpoint)
            .collect::<Vec<_>>();

        tx.declare_flow(destruction_flow(
            Asset::Ent,
            entitlement_inputs,
            Tag::Entitlement,
            settled_principal,
        ))?;

        let terminal = remaining_principal.is_zero();

        let mut vault_destinations = receipt_output_refs;

        if terminal {
            tx.declare_flow(destruction_flow(
                Asset::DistCtl,
                vec![self.control],
                Tag::DistributionControlClose,
                Sat::ONE,
            ))?;

            let residue = remaining_live.checked_add(remaining_time_locked)?;

            if let Some(vault_input) = self.vault {
                let mut destructions = Vec::new();

                if !residue.is_zero() {
                    destructions.push(DestructionLeg {
                        tag: Tag::DistributionResidue,
                        amount: residue,
                    });
                }

                // In terminal mode, one vault flow contains both
                // receipt destinations and residue destruction. The
                // vault input is cited exactly once.
                tx.declare_flow(CanonicalFlow {
                    asset: Asset::U,
                    source_inputs: vec![vault_input],
                    destination_outputs: vault_destinations,
                    destructions,
                    movement_kind: if moved.is_zero() {
                        None
                    } else {
                        Some(DeltaKind::Lateral)
                    },
                })?;
            } else if !moved.is_zero() || !residue.is_zero() {
                return Err(Guard::ControlVaultWeld);
            }
        } else {
            let control_output = tx.emit(
                Asset::DistCtl,
                Sat::ONE,
                Meta::DistributionControl {
                    cycle: control.cycle,
                    principal: control.principal,
                    live_allocation: control.live_allocation,
                    time_locked_allocation: control.time_locked_allocation,
                    remaining_principal,
                    remaining_live_value: remaining_live,
                    remaining_time_locked_value: remaining_time_locked,
                },
            );

            tx.declare_flow(movement_flow(
                Asset::DistCtl,
                DeltaKind::Lateral,
                vec![self.control],
                vec![control_output],
            ))?;

            if !successor_vault_value.is_zero() {
                let vault_output = tx.emit(
                    Asset::U,
                    successor_vault_value,
                    Meta::DistributionVault {
                        cycle: control.cycle,
                    },
                );

                vault_destinations.push(vault_output);
            }

            if let Some(vault_input) = self.vault {
                tx.declare_flow(movement_flow(
                    Asset::U,
                    DeltaKind::Lateral,
                    vec![vault_input],
                    vault_destinations,
                ))?;
            } else if !vault_destinations.is_empty() {
                return Err(Guard::ControlVaultWeld);
            }
        }

        Ok(tx.finish()?.world)
    }
}
