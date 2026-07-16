//! Cycle processing with atomic maturity conversion.
//!
//! Implements `(´def:operations:cycle-caller´)` and
//! `(´branch:operations:cycle´)`.
//!
//! The maturity-cycle CPFP anchor is modeled as a zero-value open
//! output. It does not enter the pool invariant.

use crate::asset::{Asset, Maturity, ReceiptClass};
use crate::fee::FeeEnvelope;
use crate::genesis::GENESIS_OWNER;
use crate::guard::Guard;
use crate::history::BranchKind;
use crate::kernel::{IssuanceDeclaration, OpenFlow, OpenFlowKind, TxBuilder};
use crate::object::Meta;
use crate::pool::PoolState;
use crate::scalar::{CanonicalOrder, Sat, checked_active_backing, floor_mul_div, floor_ratio};
use crate::signer::{SignerSet, require_signer};
use crate::transition::Transition;
use crate::world::World;

// ´def:operations:cycle-caller´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleCaller {
    Operator,
    Anyone,
}

// ´branch:operations:cycle´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunCycle {
    pub caller: CycleCaller,
    pub operator_signers: SignerSet,
    pub fee_envelope: FeeEnvelope,
}

impl Transition for RunCycle {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        let (state_outpoint, state) = world.state()?;

        let y = state.y()?;

        if y.is_zero() {
            return Err(Guard::Sealed);
        }

        if world.pace_age_blocks < world.constants.min_cadence_blocks {
            return Err(Guard::CadenceTooEarly);
        }

        if world.pace_age_blocks < world.constants.max_cadence_blocks
            && self.caller != CycleCaller::Operator
        {
            return Err(Guard::CadenceOperatorOnly);
        }

        if self.caller == CycleCaller::Operator {
            require_signer(&self.operator_signers, GENESIS_OWNER)?;
        }

        let (resv_outpoint, resv) = world.active_resv()?;

        let pace_outpoint = world.roots.pace;

        let distribution_authority_outpoint = world.roots.distribution_authority;

        let next_cycle = state.next_cycle()?;

        let at_maturity = matches!(
            state.maturity,
            Maturity::Announced { cycle } if cycle == next_cycle
        );

        let normal_phase = at_maturity || matches!(state.maturity, Maturity::Complete);

        let issuance = if state.q.is_zero() {
            Sat::ZERO
        } else {
            floor_mul_div(state.q, y, state.omega)?
        };

        let live_issuance = if normal_phase {
            issuance
        } else {
            floor_ratio(issuance, world.constants.zeta)?
        };

        let time_locked_issuance = issuance.checked_sub(live_issuance)?;

        let operator_live = floor_ratio(live_issuance, world.constants.mint_fee)?;

        let operator_time_locked = floor_ratio(time_locked_issuance, world.constants.mint_fee)?;

        let contributor_live = live_issuance.checked_sub(operator_live)?;

        let contributor_time_locked = time_locked_issuance.checked_sub(operator_time_locked)?;

        // Cycle preserves the active RESV value, so a cap-satisfying
        // input state is automatically preserved; this defensive check
        // is the branch-local postcondition. Q' = 0, so the successor
        // backing is exactly omega_next.
        let omega_next = checked_active_backing(state.omega, state.q)?;

        if resv.value != omega_next {
            return Err(Guard::ResvWeld);
        }

        let (y_l_next, y_t_next, maturity_next) = if at_maturity {
            (
                state
                    .y_l
                    .checked_add(live_issuance)?
                    .checked_add(state.y_t)?,
                Sat::ZERO,
                Maturity::Complete,
            )
        } else {
            (
                state.y_l.checked_add(live_issuance)?,
                state.y_t.checked_add(time_locked_issuance)?,
                state.maturity,
            )
        };

        let y_next = y_l_next.checked_add(y_t_next)?;

        if y_next > omega_next {
            return Err(Guard::OverDraw);
        }

        let mut tx = TxBuilder::new(world, BranchKind::Cycle, order);

        tx.consume(state_outpoint)?;
        tx.consume(resv_outpoint)?;
        tx.consume(pace_outpoint)?;
        tx.consume(distribution_authority_outpoint)?;

        tx.apply_fee_envelope(&self.fee_envelope)?;

        let state_output = tx.emit(
            Asset::Pid,
            Sat::ONE,
            Meta::State(PoolState {
                omega: omega_next,
                y_l: y_l_next,
                y_t: y_t_next,
                q: Sat::ZERO,
                cycle: next_cycle,
                maturity: maturity_next,
            }),
        );

        let resv_output = tx.emit(Asset::Lbtc, resv.value, Meta::Resv);

        // The active RESV value is unchanged across cycle. Optional
        // sponsor inputs fund the explicit fee. The zero-value CPFP
        // anchor is not included in an open flow.
        tx.declare_open_flow(OpenFlow {
            kind: OpenFlowKind::ReserveCarry,
            source_inputs: vec![resv_outpoint],
            destination_outputs: vec![resv_output],
            fee: Sat::ZERO,
        })?;

        let pace_output = tx.emit(Asset::Pace, Sat::ONE, Meta::Pace);

        let distribution_authority_output =
            tx.emit(Asset::DistAuth, Sat::ONE, Meta::DistributionAuthority);

        let mut receipt_destinations = Vec::new();

        if !operator_live.is_zero() {
            receipt_destinations.push(tx.emit(
                Asset::U,
                operator_live,
                Meta::Receipt {
                    owner: GENESIS_OWNER,
                    class: ReceiptClass::Live,
                },
            ));
        }

        if !operator_time_locked.is_zero() {
            // normal_phase ⟹ time_locked_issuance = 0 ⟹ operator_time_locked = 0,
            // so this receipt is always TimeLocked (bootstrapping phase only).
            receipt_destinations.push(tx.emit(
                Asset::U,
                operator_time_locked,
                Meta::Receipt {
                    owner: GENESIS_OWNER,
                    class: ReceiptClass::TimeLocked,
                },
            ));
        }

        let mut control_destination = None;

        if !state.q.is_zero() {
            let control = tx.emit(
                Asset::DistCtl,
                Sat::ONE,
                Meta::DistributionControl {
                    cycle: next_cycle,
                    principal: state.q,
                    live_allocation: contributor_live,
                    time_locked_allocation: contributor_time_locked,
                    remaining_principal: state.q,
                    remaining_live_value: contributor_live,
                    remaining_time_locked_value: contributor_time_locked,
                },
            );

            control_destination = Some(control);

            let vault_value = contributor_live.checked_add(contributor_time_locked)?;

            if !vault_value.is_zero() {
                let vault = tx.emit(
                    Asset::U,
                    vault_value,
                    Meta::DistributionVault { cycle: next_cycle },
                );

                receipt_destinations.push(vault);
            }
        }

        if at_maturity {
            tx.emit(Asset::Lbtc, Sat::ZERO, Meta::CpfpAnchor);
        }

        if !issuance.is_zero() {
            tx.declare_issuance(IssuanceDeclaration {
                asset: Asset::U,
                authority_asset: Asset::Pace,
                authority_input: pace_outpoint,
                amount: issuance,
                destination_outputs: receipt_destinations,
            })?;
        }

        if let Some(control_destination) = control_destination {
            tx.declare_issuance(IssuanceDeclaration {
                asset: Asset::DistCtl,
                authority_asset: Asset::DistAuth,
                authority_input: distribution_authority_outpoint,
                amount: Sat::ONE,
                destination_outputs: vec![control_destination],
            })?;
        }

        let result = tx.finish()?;

        let state_successor = *result
            .output_map
            .get(&state_output)
            .ok_or(Guard::MissingOutputIndex)?;

        let resv_successor = *result
            .output_map
            .get(&resv_output)
            .ok_or(Guard::MissingOutputIndex)?;

        let pace_successor = *result
            .output_map
            .get(&pace_output)
            .ok_or(Guard::MissingOutputIndex)?;

        let distribution_authority_successor = *result
            .output_map
            .get(&distribution_authority_output)
            .ok_or(Guard::MissingOutputIndex)?;

        let next = result.world;

        if next.roots.state != state_successor
            || next.roots.resv != Some(resv_successor)
            || next.roots.pace != pace_successor
            || next.roots.distribution_authority != distribution_authority_successor
        {
            return Err(Guard::RootSuccession);
        }

        // pace_age_blocks is reset by the kernel's root-cursor update
        // (the PACE successor edge owns the clock); no branch-local
        // reset is needed here.
        Ok(next)
    }
}
