//! Sponsored-maintenance action family, scheduler interface, the
//! deterministic maintenance scheduler, and the sweepability and
//! sponsored-quiescence drivers.
//!
//! Implements `´def:verification:maintenance-action´`,
//! `´def:verification:maintenance-sponsor´`,
//! `´def:verification:quiescence-scheduler´`,
//! `´rule:verification:apply-maintenance-action´`,
//! `´rule:verification:next-test-order´`,
//! `´def:verification:maintenance-mode´`,
//! `´def:verification:deterministic-maintenance-scheduler´`,
//! `´def:verification:maintenance-phase-potential´`,
//! `´rule:verification:maintenance-progress´`,
//! `´rule:verification:admissible-requests´`,
//! `´rule:verification:live-distribution-cycles´`,
//! `´rule:verification:entitlement-batch-for-cycle´`,
//! `´rule:verification:clearable-ash-batch´`,
//! `´rule:verification:compactable-ash-batch´`,
//! `´rule:verification:relabelable-receipt-batch´`,
//! `´rule:verification:deterministic-maintenance-selection´`,
//! `´thm:scalability:sponsored-quiescence´`,
//! `´thm:scalability:sweepability´` (driver forms),
//! `´def:verification:state-candidate´`, and
//! `´rule:verification:apply-state-candidate´`.
//!
//! The scheduler is a test harness, not covenant state. The
//! sweepability and quiescence drivers prove that valid collection
//! transitions exist and can drive the system to the stated residual
//! bounds when a scheduler supplies inclusion and transaction fees.
//! They do not prove that a real fee market will fund every
//! transition. The zero-fee scheduler is a proof harness for
//! reachability and collectability, not an economic claim.

use crate::asset::{Asset, Maturity, ReceiptClass};
use crate::fee::FeeEnvelope;
use crate::genesis::OPERATOR_KEY;
use crate::guard::Guard;
use crate::invariant::check_invariant;
use crate::object::Meta;
use crate::ops::{
    AdmitDeposits, AnnounceMaturity, ClearAsh, CompactAsh, CycleCaller, RedeemReceipt,
    RelabelReceipts, RunCycle, SettleDistribution,
};
use crate::quiescence::{QuiescenceReport, lifecycle_report};
use crate::recognition::{read_ash, validate_request_for_admission};
use crate::scalar::{CanonicalOrder, Cycle, OutPoint, OwnerKey, Sat};
use crate::signer::SignerSet;
use crate::transition::Transition;
use crate::world::World;

// ´def:verification:maintenance-action´

/// Every action receives a sponsor fee envelope supplied by the
/// maintenance scheduler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaintenanceAction {
    Admit {
        requests: Vec<OutPoint>,
        reward: Sat,
        reward_owner: OwnerKey,
    },

    RunCycle {
        caller: CycleCaller,
        operator_signers: SignerSet,
    },

    Settle {
        control: OutPoint,
        vault: Option<OutPoint>,
        entitlements: Vec<OutPoint>,
    },

    CompactAsh {
        ash: Vec<OutPoint>,
    },

    ClearAsh {
        ash: Vec<OutPoint>,
    },

    Relabel {
        receipts: Vec<OutPoint>,
    },
}

// ´def:verification:maintenance-sponsor´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaintenanceSponsor {
    pub owner: OwnerKey,
    pub available_inputs: Vec<OutPoint>,
    pub signers: SignerSet,

    pub fee_per_action: Sat,
}

// ´def:verification:quiescence-scheduler´

pub trait MaintenanceScheduler {
    fn next_action(&self, world: &World) -> Result<Option<MaintenanceAction>, Guard>;

    fn fee_envelope_for(
        &self,
        world: &World,
        action: &MaintenanceAction,
    ) -> Result<FeeEnvelope, Guard>;
}

// ´thm:scalability:sweepability´ (driver form)

pub fn drive_shared_state_to_fixpoint<S>(
    initial: &World,
    scheduler: &S,
    mut next_order: impl FnMut() -> CanonicalOrder,
) -> Result<World, Guard>
where
    S: MaintenanceScheduler,
{
    let mut world = initial.clone();

    while let Some(action) = scheduler.next_action(&world)? {
        let potential_before = maintenance_phase_potential(&world)?;

        let envelope = scheduler.fee_envelope_for(&world, &action)?;

        world = apply_maintenance_action(&world, action, envelope, next_order())?;

        check_invariant(&world).map_err(|_| Guard::InvariantFailure)?;

        require_maintenance_progress(&world, potential_before)?;
    }

    Ok(world)
}

// ´def:verification:maintenance-phase-potential´

/// Lexicographic phase potential over the maintenance state.
///
/// Every legitimate maintenance action strictly decreases it: earlier
/// phases may repopulate later components (admission enables a cycle,
/// a cycle creates a distribution, settlement creates receipts), but
/// each action strictly decreases the first component it affects.
pub fn maintenance_phase_potential(world: &World) -> Result<[u64; 7], Guard> {
    let state = world.state()?.1;

    let report = lifecycle_report(world)?;

    let relabelable = if state.maturity == Maturity::Complete {
        report.time_locked_receipts as u64
    } else {
        0
    };

    let ash_total = find_ash(world)
        .into_iter()
        .map(|outpoint| read_ash(world.utxo(outpoint)?))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    let y = state.y()?;

    let clearable = if y <= Sat::ONE {
        Sat::ZERO
    } else {
        ash_total.min(state.y_l).min(y.checked_sub(Sat::ONE)?)
    };

    Ok([
        report.admissible_requests as u64,
        u64::from(!state.q.is_zero()),
        report.entitlements as u64,
        report.live_distributions as u64,
        relabelable,
        report.ash_outputs as u64,
        clearable.get(),
    ])
}

// ´rule:verification:maintenance-progress´

/// Rejects scheduler loops: an applied maintenance action that does
/// not strictly decrease the phase potential indicates a broken
/// scheduler cycling without progress, and is reported immediately
/// rather than silently consuming the step budget.
fn require_maintenance_progress(world: &World, potential_before: [u64; 7]) -> Result<(), Guard> {
    if maintenance_phase_potential(world)? >= potential_before {
        return Err(Guard::ZeroProgress);
    }

    Ok(())
}

// ´rule:verification:apply-maintenance-action´
//
// Admission uses request-internal budgets, not the generic sponsor
// envelope. The scheduler's envelope for admission is ignored by the
// current branch design and should be empty.

pub fn apply_maintenance_action(
    world: &World,
    action: MaintenanceAction,
    fee_envelope: FeeEnvelope,
    order: CanonicalOrder,
) -> Result<World, Guard> {
    match action {
        MaintenanceAction::Admit {
            requests,
            reward,
            reward_owner,
        } => AdmitDeposits {
            requests,
            admission_reward: reward,
            reward_owner,
        }
        .apply(world, order),

        MaintenanceAction::RunCycle {
            caller,
            operator_signers,
        } => RunCycle {
            caller,
            operator_signers,
            fee_envelope,
        }
        .apply(world, order),

        MaintenanceAction::Settle {
            control,
            vault,
            entitlements,
        } => SettleDistribution {
            control,
            vault,
            entitlements,
            fee_envelope,
        }
        .apply(world, order),

        MaintenanceAction::CompactAsh { ash } => CompactAsh {
            ash_inputs: ash,
            fee_envelope,
        }
        .apply(world, order),

        MaintenanceAction::ClearAsh { ash } => ClearAsh {
            ash_inputs: ash,
            fee_envelope,
        }
        .apply(world, order),

        MaintenanceAction::Relabel { receipts } => RelabelReceipts {
            receipts,
            fee_envelope,
        }
        .apply(world, order),
    }
}

// ´rule:verification:next-test-order´

/// This helper is for the deterministic oracle harness. Deployment
/// order comes from actual block height and transaction index.
pub fn next_model_order(world: &World) -> Result<CanonicalOrder, Guard> {
    let previous = world.history.last_order();

    if previous.tx_index == u32::MAX {
        Ok(CanonicalOrder {
            height: previous.height.checked_add(1).ok_or(Guard::Overflow)?,
            tx_index: 0,
        })
    } else {
        Ok(CanonicalOrder {
            height: previous.height,
            tx_index: previous.tx_index + 1,
        })
    }
}

// ´def:verification:maintenance-mode´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaintenanceMode {
    // Assumes no pending admissible requests and Q = 0.
    SharedStateOnly,

    // Admits requests, runs required cycles, and then
    // performs shared-state maintenance.
    FullSponsored,
}

// ´def:verification:deterministic-maintenance-scheduler´

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeterministicMaintenanceScheduler {
    pub mode: MaintenanceMode,
}

// Canonical-object lookups for the deterministic harness.

pub fn find_ash(world: &World) -> Vec<OutPoint> {
    world
        .utxos
        .iter()
        .filter_map(|(outpoint, utxo)| {
            if matches!((utxo.asset, utxo.meta), (Asset::U, Meta::Ash)) {
                Some(*outpoint)
            } else {
                None
            }
        })
        .collect()
}

/// Harness lookup: callers guarantee the control exists (for example
/// via [`live_distribution_cycles`]).
pub fn find_distribution_control(world: &World, cycle: Cycle) -> OutPoint {
    world
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (
                Asset::DistCtl,
                Meta::DistributionControl {
                    cycle: control_cycle,
                    ..
                },
            ) if control_cycle == cycle => Some(*outpoint),

            _ => None,
        })
        .expect("distribution control")
}

pub fn find_distribution_vault(world: &World, cycle: Cycle) -> Option<OutPoint> {
    world
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (Asset::U, Meta::DistributionVault { cycle: vault_cycle }) if vault_cycle == cycle => {
                Some(*outpoint)
            }

            _ => None,
        })
}

// ´rule:verification:admissible-requests´

pub fn admissible_requests(world: &World) -> Vec<OutPoint> {
    world
        .utxos
        .iter()
        .filter_map(|(outpoint, utxo)| {
            if validate_request_for_admission(&world.constants, utxo).is_ok() {
                Some(*outpoint)
            } else {
                None
            }
        })
        .take(world.constants.admission_batch_max)
        .collect()
}

// ´rule:verification:live-distribution-cycles´

pub fn live_distribution_cycles(world: &World) -> Vec<Cycle> {
    let mut cycles = world
        .utxos
        .values()
        .filter_map(|utxo| match (utxo.asset, utxo.meta) {
            (Asset::DistCtl, Meta::DistributionControl { cycle, .. }) => Some(cycle),

            _ => None,
        })
        .collect::<Vec<_>>();

    cycles.sort_unstable();
    cycles.dedup();

    cycles
}

// ´rule:verification:entitlement-batch-for-cycle´

pub fn entitlement_batch_for_cycle(world: &World, cycle: Cycle) -> Vec<OutPoint> {
    world
        .utxos
        .iter()
        .filter_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (Asset::Ent, Meta::DepositEntitlement { target_cycle, .. })
                if target_cycle == cycle =>
            {
                Some(*outpoint)
            }

            _ => None,
        })
        .take(world.constants.settlement_batch_max)
        .collect()
}

// ´rule:verification:clearable-ash-batch´

pub(crate) fn clearable_ash_batch(world: &World) -> Result<Option<Vec<OutPoint>>, Guard> {
    let state = world.state()?.1;
    let y = state.y()?;

    if y <= Sat::ONE {
        return Ok(None);
    }

    let batch = find_ash(world)
        .into_iter()
        .take(world.constants.ash_batch_max)
        .collect::<Vec<_>>();

    if batch.is_empty() {
        return Ok(None);
    }

    let total = batch
        .iter()
        .map(|outpoint| read_ash(world.utxo(*outpoint)?))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .try_fold(Sat::ZERO, |acc, value| acc.checked_add(value))?;

    let clear_amount = total.min(state.y_l).min(y.checked_sub(Sat::ONE)?);

    if clear_amount.is_zero() {
        Ok(None)
    } else {
        Ok(Some(batch))
    }
}

// ´rule:verification:compactable-ash-batch´

fn compactable_ash_batch(world: &World) -> Option<Vec<OutPoint>> {
    let batch = find_ash(world)
        .into_iter()
        .take(world.constants.ash_batch_max)
        .collect::<Vec<_>>();

    if batch.len() >= 2 { Some(batch) } else { None }
}

// ´rule:verification:relabelable-receipt-batch´

fn relabelable_receipt_batch(world: &World) -> Result<Option<Vec<OutPoint>>, Guard> {
    let state = world.state()?.1;

    if state.maturity != Maturity::Complete {
        return Ok(None);
    }

    let batch = world
        .utxos
        .iter()
        .filter_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (
                Asset::U,
                Meta::Receipt {
                    class: ReceiptClass::TimeLocked,
                    ..
                },
            ) => Some(*outpoint),

            _ => None,
        })
        .take(world.constants.relabel_batch_max)
        .collect::<Vec<_>>();

    if batch.is_empty() {
        Ok(None)
    } else {
        Ok(Some(batch))
    }
}

// ´rule:verification:deterministic-maintenance-selection´

impl MaintenanceScheduler for DeterministicMaintenanceScheduler {
    fn next_action(&self, world: &World) -> Result<Option<MaintenanceAction>, Guard> {
        if self.mode == MaintenanceMode::FullSponsored {
            let requests = admissible_requests(world);

            if !requests.is_empty() {
                return Ok(Some(MaintenanceAction::Admit {
                    requests,
                    reward: Sat::ZERO,
                    reward_owner: OPERATOR_KEY,
                }));
            }

            let state = world.state()?.1;

            if !state.q.is_zero() {
                return Ok(Some(MaintenanceAction::RunCycle {
                    caller: CycleCaller::Anyone,
                    operator_signers: SignerSet::new(),
                }));
            }
        }

        for cycle in live_distribution_cycles(world) {
            let entitlements = entitlement_batch_for_cycle(world, cycle);

            if entitlements.is_empty() {
                continue;
            }

            let control = find_distribution_control(world, cycle);

            let vault = find_distribution_vault(world, cycle);

            return Ok(Some(MaintenanceAction::Settle {
                control,
                vault,
                entitlements,
            }));
        }

        if let Some(receipts) = relabelable_receipt_batch(world)? {
            return Ok(Some(MaintenanceAction::Relabel { receipts }));
        }

        if let Some(batch) = compactable_ash_batch(world) {
            return Ok(Some(MaintenanceAction::CompactAsh { ash: batch }));
        }

        if let Some(batch) = clearable_ash_batch(world)? {
            return Ok(Some(MaintenanceAction::ClearAsh { ash: batch }));
        }

        Ok(None)
    }

    fn fee_envelope_for(
        &self,
        _world: &World,
        action: &MaintenanceAction,
    ) -> Result<FeeEnvelope, Guard> {
        match action {
            // Admission uses request-internal budgets.
            MaintenanceAction::Admit { .. } => Ok(FeeEnvelope::default()),

            // The deterministic oracle abstracts a sponsor
            // willing to include zero-fee model transitions.
            // Deployment tests separately exercise real fee inputs.
            _ => Ok(FeeEnvelope::default()),
        }
    }
}

// ´thm:scalability:sponsored-quiescence´

/// Generic scheduler-driven quiescence loop with progress detection.
///
/// `maximum_steps` bounds a scheduler that keeps producing productive
/// actions; a scheduler that repeats a no-progress action is rejected
/// immediately with [`Guard::ZeroProgress`].
pub fn drive_quiescence_with_scheduler<S>(
    initial: &World,
    scheduler: &S,
    maximum_steps: usize,
    force_cadence: bool,
) -> Result<(World, QuiescenceReport), Guard>
where
    S: MaintenanceScheduler,
{
    let mut world = initial.clone();

    for _ in 0..maximum_steps {
        let Some(action) = scheduler.next_action(&world)? else {
            let report = lifecycle_report(&world)?;

            return Ok((world, report));
        };

        if force_cadence && matches!(action, MaintenanceAction::RunCycle { .. }) {
            world.pace_age_blocks = world.constants.max_cadence_blocks;
        }

        let potential_before = maintenance_phase_potential(&world)?;

        let envelope = scheduler.fee_envelope_for(&world, &action)?;

        world = apply_maintenance_action(&world, action, envelope, next_model_order(&world)?)?;

        check_invariant(&world).map_err(|_| Guard::InvariantFailure)?;

        require_maintenance_progress(&world, potential_before)?;
    }

    Err(Guard::Overflow)
}

/// `maximum_steps` prevents a buggy scheduler from looping forever in
/// the test harness.
pub fn drive_sponsored_quiescence(
    initial: &World,
    maximum_steps: usize,
) -> Result<(World, QuiescenceReport), Guard> {
    let scheduler = DeterministicMaintenanceScheduler {
        mode: MaintenanceMode::FullSponsored,
    };

    drive_quiescence_with_scheduler(initial, &scheduler, maximum_steps, true)
}

// ´thm:scalability:sweepability´ (shared-state driver)

pub fn drive_shared_state_sweepability(
    initial: &World,
    maximum_steps: usize,
) -> Result<(World, QuiescenceReport), Guard> {
    let state = initial.state()?.1;

    if !state.q.is_zero() || !admissible_requests(initial).is_empty() {
        return Err(Guard::BadAuthorization);
    }

    let scheduler = DeterministicMaintenanceScheduler {
        mode: MaintenanceMode::SharedStateOnly,
    };

    drive_quiescence_with_scheduler(initial, &scheduler, maximum_steps, false)
}

// ´def:verification:state-candidate´

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateCandidate {
    Admit(AdmitDeposits),
    Cycle(RunCycle),
    Redeem(RedeemReceipt),
    Relabel(RelabelReceipts),
    Clear(ClearAsh),
    Announce(AnnounceMaturity),
}

// ´rule:verification:apply-state-candidate´

impl StateCandidate {
    pub fn apply(&self, world: &World, order: CanonicalOrder) -> Result<World, Guard> {
        match self {
            StateCandidate::Admit(transition) => transition.apply(world, order),

            StateCandidate::Cycle(transition) => transition.apply(world, order),

            StateCandidate::Redeem(transition) => transition.apply(world, order),

            StateCandidate::Relabel(transition) => transition.apply(world, order),

            StateCandidate::Clear(transition) => transition.apply(world, order),

            StateCandidate::Announce(transition) => transition.apply(world, order),
        }
    }
}
