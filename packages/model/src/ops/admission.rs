//! Bounded-batch deposit admission.
//!
//! Implements `(´branch:operations:admit-deposits´)`.
//!
//! Admission emits exactly one entitlement per request. It never
//! aggregates entitlements by owner or target.

use crate::asset::Asset;
use crate::guard::Guard;
use crate::history::BranchKind;
use crate::kernel::{IssuanceDeclaration, OpenFlow, OpenFlowKind, TxBuilder, ensure_distinct};
use crate::object::Meta;
use crate::pool::PoolState;
use crate::recognition::validate_request_for_admission;
use crate::scalar::{CanonicalOrder, OutPoint, OwnerKey, Sat, checked_active_backing};
use crate::transition::Transition;
use crate::world::World;

// ´branch:operations:admit-deposits´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmitDeposits {
    pub requests: Vec<OutPoint>,

    /// Admitter-chosen reward, valid only **within** the preauthorized
    /// admission budget carried by the consumed requests
    /// (`admission_reward <= budget_total`, enforced below). No pool
    /// quantity depends on it — the remainder funds the chain fee — so
    /// this is service compensation inside a depositor-authorized
    /// envelope, not numeric discretion over the economy.
    pub admission_reward: Sat,
    pub reward_owner: OwnerKey,
}

impl Transition for AdmitDeposits {
    fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        if self.requests.is_empty() || self.requests.len() > world.constants.admission_batch_max {
            return Err(Guard::Domain);
        }

        ensure_distinct(&self.requests)?;

        let (state_outpoint, state) = world.state()?;

        if state.y()?.is_zero() {
            return Err(Guard::Sealed);
        }

        let (resv_outpoint, resv) = world.active_resv()?;

        let entitlement_authority_outpoint = world.roots.entitlement_authority;

        let mut views = Vec::new();

        let mut deposit_total = Sat::ZERO;
        let mut budget_total = Sat::ZERO;

        for request_outpoint in &self.requests {
            let request = world.utxo(*request_outpoint)?;

            let view = validate_request_for_admission(&world.constants, request)?;

            deposit_total = deposit_total.checked_add(view.deposit_principal)?;

            budget_total = budget_total.checked_add(view.admission_budget)?;

            views.push((*request_outpoint, view));
        }

        let successor_reserve_value = resv.value.checked_add(deposit_total)?;

        let successor_q = state.q.checked_add(deposit_total)?;

        // Ω + (Q + D) <= ACTIVE_BACKING_MAX: admission fails closed when its
        // batch exceeds remaining active-backing headroom. Each
        // request remains independently admissible in a smaller batch,
        // later after redemption, or cancelable.
        let successor_backing = checked_active_backing(state.omega, successor_q)?;

        if successor_reserve_value != successor_backing {
            return Err(Guard::ResvWeld);
        }

        if self.admission_reward > budget_total {
            return Err(Guard::PartitionPin);
        }

        let chain_fee = budget_total.checked_sub(self.admission_reward)?;

        let target_cycle = state.next_cycle()?;

        let mut tx = TxBuilder::new(world, BranchKind::AdmitDeposits, order);

        tx.consume(state_outpoint)?;
        tx.consume(resv_outpoint)?;
        tx.consume(entitlement_authority_outpoint)?;

        for (request_outpoint, _) in &views {
            tx.consume(*request_outpoint)?;
        }

        let state_output = tx.emit(
            Asset::Pid,
            Sat::ONE,
            Meta::State(PoolState {
                q: successor_q,
                ..state
            }),
        );

        let resv_output = tx.emit(Asset::Lbtc, successor_reserve_value, Meta::Resv);

        let entitlement_authority_output =
            tx.emit(Asset::EntAuth, Sat::ONE, Meta::EntitlementAuthority);

        let mut entitlement_outputs = Vec::new();

        for (_request_outpoint, view) in &views {
            let output = tx.emit(
                Asset::Ent,
                view.deposit_principal,
                Meta::DepositEntitlement {
                    owner: view.receipt_owner,
                    target_cycle,
                },
            );

            entitlement_outputs.push(output);
        }

        tx.declare_issuance(IssuanceDeclaration {
            asset: Asset::Ent,
            authority_asset: Asset::EntAuth,
            authority_input: entitlement_authority_outpoint,
            amount: deposit_total,
            destination_outputs: entitlement_outputs,
        })?;

        let mut admission_sources = vec![resv_outpoint];

        admission_sources.extend(self.requests.iter().copied());

        let mut admission_destinations = vec![resv_output];

        if !self.admission_reward.is_zero() {
            let reward = tx.emit(
                Asset::Lbtc,
                self.admission_reward,
                Meta::PlainLbtc {
                    owner: self.reward_owner,
                },
            );

            admission_destinations.push(reward);
        }

        tx.declare_open_flow(OpenFlow {
            kind: OpenFlowKind::DepositAdmission,
            source_inputs: admission_sources,
            destination_outputs: admission_destinations,
            fee: chain_fee,
        })?;

        tx.set_chain_fee(chain_fee);

        let result = tx.finish()?;

        let state_successor = *result
            .output_map
            .get(&state_output)
            .ok_or(Guard::MissingOutputIndex)?;

        let resv_successor = *result
            .output_map
            .get(&resv_output)
            .ok_or(Guard::MissingOutputIndex)?;

        let entitlement_authority_successor = *result
            .output_map
            .get(&entitlement_authority_output)
            .ok_or(Guard::MissingOutputIndex)?;

        let next = result.world;

        if next.roots.state != state_successor
            || next.roots.resv != Some(resv_successor)
            || next.roots.entitlement_authority != entitlement_authority_successor
        {
            return Err(Guard::RootSuccession);
        }

        Ok(next)
    }
}
