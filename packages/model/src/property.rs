//! Property-test action model and state-aware executor.
//!
//! Implements `´def:verification:property-action´`,
//! `´def:verification:property-step-result´`,
//! `´lst:verification:property-owner-helpers´`,
//! `´rule:verification:property-world-funding´`,
//! `´rule:verification:property-selectors´`,
//! `´def:verification:property-action-seed´`,
//! `´rule:verification:property-create-request´`,
//! `´rule:verification:property-cancel-request´`,
//! `´rule:verification:property-admission´`,
//! `´rule:verification:property-cycle´`,
//! `´rule:verification:property-settlement´`,
//! `´rule:verification:property-transfer´`,
//! `´rule:verification:property-redemption´`,
//! `´rule:verification:property-relabel´`,
//! `´rule:verification:property-burn´`,
//! `´rule:verification:property-ash-compaction´`,
//! `´rule:verification:property-clear´`,
//! `´rule:verification:property-announce-maturity´`,
//! `´rule:verification:property-open-injection´`,
//! `´rule:verification:property-step´`,
//! `´rule:verification:materialize-action´`,
//! `´rule:verification:property-seed-trace´`, and
//! `´rule:verification:property-trace´`.
//!
//! The generated values remain far below the `Sat` ceiling; dedicated
//! boundary tests cover near-ceiling arithmetic separately. The
//! `CreateRequest` arm preserves the exact generated `principal`,
//! `budget`, and `owner`; no property amount narrows through `u16`.
//!
//! One may assume naively that the executor could inline the cycle
//! constructor and omit a cancellation arm, but that leaves the
//! dedicated cancel and cycle helpers unused (`deny(unused)`); the
//! executor therefore routes `CancelSomeRequest` through
//! `property_cancel_request` and `RunCycle` through
//! `property_run_cycle`.

use crate::asset::{Asset, Maturity, ReceiptClass};
use crate::fee::{FeeChange, FeeEnvelope};
use crate::genesis::OPERATOR_KEY;
use crate::guard::Guard;
use crate::history::BurnRecord;
use crate::invariant::check_invariant;
use crate::maintenance::{
    admissible_requests, clearable_ash_batch, entitlement_batch_for_cycle, find_ash,
    find_distribution_control, find_distribution_vault, live_distribution_cycles, next_model_order,
};
use crate::object::Meta;
use crate::ops::{
    AdmitDeposits, AnnounceMaturity, BurnReceipts, CancelRequest, ClearAsh, CompactAsh,
    CreateRequest, CycleCaller, ReceiptDestination, RedeemReceipt, RelabelReceipts, RunCycle,
    SettleDistribution, TransferReceipts, inject_open_object,
};
use crate::pool::PoolState;
use crate::recognition::{ReceiptView, read_receipt, validate_request_for_admission};
use crate::scalar::{
    ACTIVE_BACKING_MAX, AttestationAddress, Cycle, OutPoint, OwnerKey, Sat, TWO_51,
};
use crate::signer::SignerSet;
use crate::transition::Transition;
use crate::world::World;

// ´lst:verification:property-owner-helpers´

pub const PROPERTY_OWNER_A: OwnerKey = OwnerKey([21_u8; 32]);

pub const PROPERTY_OWNER_B: OwnerKey = OwnerKey([22_u8; 32]);

pub const PROPERTY_OWNER_C: OwnerKey = OwnerKey([23_u8; 32]);

pub const PROPERTY_SPONSOR: OwnerKey = OwnerKey([24_u8; 32]);

pub const PROPERTY_ADDRESS_A: AttestationAddress = AttestationAddress([31_u8; 32]);

pub const PROPERTY_ADDRESS_B: AttestationAddress = AttestationAddress([32_u8; 32]);

/// General trace amounts remain intentionally modest so random traces
/// produce many accepted transitions. Near-`2^51` arithmetic is covered
/// by dedicated boundary tests.
const PROPERTY_PRINCIPAL_MAX: u64 = 65_536;
const PROPERTY_BUDGET_MAX: u64 = 4_096;
const PROPERTY_FOREIGN_ASSET_ID: u32 = 0xF001;

fn projected_positive(raw: u64, maximum_inclusive: u64) -> Result<Sat, Guard> {
    if maximum_inclusive == 0 {
        return Err(Guard::BadConstant);
    }

    let value = 1 + (raw % maximum_inclusive);

    Sat::positive(value)
}

fn projected_principal(raw: u64) -> Result<Sat, Guard> {
    projected_positive(raw, PROPERTY_PRINCIPAL_MAX)
}

fn projected_budget(raw: u64) -> Result<Sat, Guard> {
    projected_positive(raw, PROPERTY_BUDGET_MAX)
}

// ´rule:verification:property-world-funding´

/// The injected outputs are ordinary open L-BTC wallet positions. They
/// remain outside the safety state until consumed by a request
/// constructor or fee envelope.
///
/// The adversarial L-BTC budget is capped at the physically plausible
/// external supply: `ACTIVE_BACKING_MAX` minus the current active reserve.
/// The cap constrains only the pool's active reserve; the retained
/// budget still funds decoy RESV, malformed requests, and anchor junk.
/// Foreign assets are not constrained by the L-BTC reserve cap.
pub fn fund_property_world(initial: &World, per_owner: Sat) -> Result<World, Guard> {
    let owners = [
        PROPERTY_OWNER_A,
        PROPERTY_OWNER_B,
        PROPERTY_OWNER_C,
        PROPERTY_SPONSOR,
    ];

    let wallet_total = owners
        .iter()
        .try_fold(Sat::ZERO, |acc, _| acc.checked_add(per_owner))?;

    let active_reserve = initial.active_resv()?.1.value;

    let physical_external_headroom = Sat::new(ACTIVE_BACKING_MAX)?.checked_sub(active_reserve)?;

    if wallet_total > physical_external_headroom {
        return Err(Guard::ActiveBackingCapExceeded);
    }

    let retained_attack_budget = physical_external_headroom.checked_sub(wallet_total)?;

    let mut world = initial.clone();

    world.adversary.lbtc = wallet_total.checked_add(retained_attack_budget)?;

    for owner in owners {
        world = inject_open_object(&world, Asset::Lbtc, per_owner, Meta::PlainLbtc { owner })?;
    }

    world
        .adversary
        .foreign
        .insert(PROPERTY_FOREIGN_ASSET_ID, Sat::new(TWO_51 - 1)?);

    Ok(world)
}

// ´rule:verification:property-selectors´

fn first_plain_lbtc(world: &World, owner: OwnerKey, minimum: Sat) -> Option<OutPoint> {
    world
        .utxos
        .iter()
        .find_map(|(outpoint, utxo)| match (utxo.asset, utxo.meta) {
            (
                Asset::Lbtc,
                Meta::PlainLbtc {
                    owner: output_owner,
                },
            ) if output_owner == owner && utxo.value >= minimum => Some(*outpoint),

            _ => None,
        })
}

fn first_receipt(world: &World, class: ReceiptClass) -> Option<(OutPoint, ReceiptView)> {
    world.utxos.iter().find_map(|(outpoint, utxo)| {
        read_receipt(utxo)
            .ok()
            .filter(|receipt| receipt.class == class)
            .map(|receipt| (*outpoint, receipt))
    })
}

fn first_live_receipt_not_exceeding(
    world: &World,
    maximum: Sat,
) -> Option<(OutPoint, ReceiptView)> {
    world.utxos.iter().find_map(|(outpoint, utxo)| {
        read_receipt(utxo)
            .ok()
            .filter(|receipt| receipt.class == ReceiptClass::Live && receipt.value <= maximum)
            .map(|receipt| (*outpoint, receipt))
    })
}

fn first_admissible_request(world: &World) -> Option<OutPoint> {
    world.utxos.iter().find_map(|(outpoint, utxo)| {
        validate_request_for_admission(&world.constants, utxo)
            .ok()
            .map(|_| *outpoint)
    })
}

#[allow(clippy::type_complexity)]
fn first_live_distribution(
    world: &World,
) -> Option<(Cycle, OutPoint, Option<OutPoint>, Vec<OutPoint>)> {
    for cycle in live_distribution_cycles(world) {
        let entitlements = entitlement_batch_for_cycle(world, cycle);

        if entitlements.is_empty() {
            continue;
        }

        return Some((
            cycle,
            find_distribution_control(world, cycle),
            find_distribution_vault(world, cycle),
            entitlements,
        ));
    }

    None
}

// ´def:verification:property-action-seed´

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropertyActionSeed {
    /// Selects one of the complete property-action families.
    pub selector: u8,

    /// Full-width generated values. They are reduced through checked
    /// modular projections only where the selected action requires a
    /// bounded amount. No `u16` truncation is permitted.
    pub amount_a: u64,
    pub amount_b: u64,

    pub owner_selector: u8,

    /// Selects adversarial open-object shapes independently from the
    /// operation selector.
    pub shape_selector: u8,
}

impl PropertyActionSeed {
    pub fn owner(self) -> OwnerKey {
        match self.owner_selector % 3 {
            0 => PROPERTY_OWNER_A,
            1 => PROPERTY_OWNER_B,
            _ => PROPERTY_OWNER_C,
        }
    }

    pub fn other_owner(self) -> OwnerKey {
        match self.owner_selector % 3 {
            0 => PROPERTY_OWNER_B,
            1 => PROPERTY_OWNER_C,
            _ => PROPERTY_OWNER_A,
        }
    }

    pub fn third_owner(self) -> OwnerKey {
        match self.owner_selector % 3 {
            0 => PROPERTY_OWNER_C,
            1 => PROPERTY_OWNER_A,
            _ => PROPERTY_OWNER_B,
        }
    }
}

// ´rule:verification:property-create-request´

fn property_create_request(
    world: &World,
    funder: OwnerKey,
    receipt_owner: OwnerKey,
    principal: Sat,
    budget: Sat,
) -> Result<World, Guard> {
    let gross = principal.checked_add(budget)?;

    let funding_input = first_plain_lbtc(world, funder, gross).ok_or(Guard::NoSuch)?;

    let input_value = world.utxo(funding_input)?.value;

    let change_value = input_value.checked_sub(gross)?;

    let change = if change_value.is_zero() {
        None
    } else {
        Some(FeeChange {
            owner: funder,
            value: change_value,
        })
    };

    CreateRequest {
        funder,
        funding_inputs: vec![funding_input],
        signers: std::iter::once(funder).collect(),

        refund_key: funder,
        receipt_owner,

        deposit_principal: principal,
        gross_value: gross,

        change,
        chain_fee: Sat::ZERO,
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-cancel-request´

fn property_cancel_request(world: &World) -> Result<World, Guard> {
    let request = first_admissible_request(world).ok_or(Guard::NoSuch)?;

    let request_utxo = world.utxo(request)?;

    let refund_key = match request_utxo.meta {
        Meta::DepositRequest { refund_key, .. } => refund_key,

        _ => return Err(Guard::WrongShape),
    };

    CancelRequest {
        request,
        signers: std::iter::once(refund_key).collect(),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-admission´

fn property_admit_requests(world: &World) -> Result<World, Guard> {
    let requests = admissible_requests(world);

    if requests.is_empty() {
        return Err(Guard::NoSuch);
    }

    AdmitDeposits {
        requests,
        admission_reward: Sat::ZERO,
        reward_owner: PROPERTY_SPONSOR,
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-cycle´

fn property_run_cycle(world: &World, caller: CycleCaller) -> Result<World, Guard> {
    let operator_signers = if caller == CycleCaller::Operator {
        std::iter::once(OPERATOR_KEY).collect()
    } else {
        SignerSet::new()
    };

    RunCycle {
        caller,
        operator_signers,
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-settlement´

fn property_settle_distribution(world: &World) -> Result<World, Guard> {
    let (_cycle, control, vault, entitlements) =
        first_live_distribution(world).ok_or(Guard::NoSuch)?;

    SettleDistribution {
        control,
        vault,
        entitlements,
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-transfer´

fn property_transfer(
    world: &World,
    class: ReceiptClass,
    destination_owner: OwnerKey,
) -> Result<World, Guard> {
    let (input, receipt) = first_receipt(world, class).ok_or(Guard::NoSuch)?;

    TransferReceipts {
        class,
        inputs: vec![input],
        outputs: vec![ReceiptDestination {
            owner: destination_owner,
            value: receipt.value,
        }],
        signers: std::iter::once(receipt.owner).collect(),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-redemption´

fn property_redeem(world: &World) -> Result<World, Guard> {
    let state = world.state()?.1;

    let (receipt, view) =
        first_live_receipt_not_exceeding(world, state.y_l).ok_or(Guard::NoSuch)?;

    RedeemReceipt {
        receipt,
        signers: std::iter::once(view.owner).collect(),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-relabel´

fn property_relabel(world: &World) -> Result<World, Guard> {
    let state = world.state()?.1;

    if state.maturity != Maturity::Complete {
        return Err(Guard::MaturityNotComplete);
    }

    let receipts = world
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

    if receipts.is_empty() {
        return Err(Guard::NoSuch);
    }

    RelabelReceipts {
        receipts,
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-burn´
//
// Every burn parameter is generated: the consumed receipt, the burned
// amount, the record claim (including over-claims, which remain valid
// transitions gated only by the off-chain indexer), the record count,
// and the attestation address.

fn live_receipts(world: &World) -> Vec<(OutPoint, ReceiptView)> {
    world
        .utxos
        .iter()
        .filter_map(|(outpoint, utxo)| {
            read_receipt(utxo)
                .ok()
                .filter(|receipt| receipt.class == ReceiptClass::Live)
                .map(|receipt| (*outpoint, receipt))
        })
        .collect()
}

fn select_address(address_selector: u8) -> AttestationAddress {
    if address_selector.is_multiple_of(2) {
        PROPERTY_ADDRESS_A
    } else {
        PROPERTY_ADDRESS_B
    }
}

fn property_burn(
    world: &World,
    owner_selector: u8,
    burn_selector: u64,
    record_selector: u64,
    address_selector: u8,
) -> Result<World, Guard> {
    let candidates = live_receipts(world);

    if candidates.is_empty() {
        return Err(Guard::NoSuch);
    }

    // Multi-owner burns consume two receipts; all owners must sign.
    let multi_input = burn_selector % 4 == 3 && candidates.len() >= 2;

    let first_index = usize::from(owner_selector) % candidates.len();

    let (receipts, signers, input_total, change_owner) = if multi_input {
        let second_index = (first_index + 1) % candidates.len();

        let (first_outpoint, first) = candidates[first_index];
        let (second_outpoint, second) = candidates[second_index];

        let total = first.value.checked_add(second.value)?;

        let signers: SignerSet = [first.owner, second.owner].into_iter().collect();

        (
            vec![first_outpoint, second_outpoint],
            signers,
            total,
            first.owner,
        )
    } else {
        let (outpoint, receipt) = candidates[first_index];

        (
            vec![outpoint],
            std::iter::once(receipt.owner).collect(),
            receipt.value,
            receipt.owner,
        )
    };

    if input_total.is_zero() {
        return Err(Guard::ZeroProgress);
    }

    let requested = 1 + (burn_selector % input_total.get());

    let ash_value = Sat::positive(requested)?;

    let change_value = input_total.checked_sub(ash_value)?;

    let change = if change_value.is_zero() {
        Vec::new()
    } else {
        vec![ReceiptDestination {
            owner: change_owner,
            value: change_value,
        }]
    };

    // Claims range over no-claim, under-claim, exact claim, and
    // over-claim (up to twice the ASH value). Over-claiming burns
    // remain valid transitions; the reference indexer rejects their
    // records off-chain.
    let claim_total = record_selector % (2 * ash_value.get() + 1);

    let address = select_address(address_selector);

    let records = if claim_total == 0 {
        Vec::new()
    } else if address_selector % 4 < 2 || claim_total < 2 {
        vec![BurnRecord {
            record_index: 0,
            address,
            amount: Sat::new(claim_total)?,
        }]
    } else {
        // Split the claim across two records at distinct addresses.
        let first_amount = claim_total / 2;
        let second_amount = claim_total - first_amount;

        vec![
            BurnRecord {
                record_index: 0,
                address,
                amount: Sat::new(first_amount)?,
            },
            BurnRecord {
                record_index: 1,
                address: select_address(address_selector.wrapping_add(1)),
                amount: Sat::new(second_amount)?,
            },
        ]
    };

    BurnReceipts {
        receipts,
        signers,

        ash_value,
        change,
        records,

        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-ash-compaction´

fn property_compact_ash(world: &World) -> Result<World, Guard> {
    let ash = find_ash(world)
        .into_iter()
        .take(world.constants.ash_batch_max)
        .collect::<Vec<_>>();

    if ash.len() < 2 {
        return Err(Guard::NoSuch);
    }

    CompactAsh {
        ash_inputs: ash,
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-clear´

fn property_clear(world: &World) -> Result<World, Guard> {
    let ash = clearable_ash_batch(world)?.ok_or(Guard::NoSuch)?;

    ClearAsh {
        ash_inputs: ash,
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-announce-maturity´

fn property_announce_maturity(world: &World, seed: PropertyActionSeed) -> Result<World, Guard> {
    let state = world.state()?.1;

    if state.maturity != Maturity::Unannounced {
        return Err(Guard::MaturityAlreadyAnnounced);
    }

    let span = world
        .constants
        .max_maturity_lead
        .checked_sub(world.constants.min_maturity_lead)
        .ok_or(Guard::CycleOverflow)?;

    let offset = if span == 0 {
        0
    } else {
        seed.amount_a % (span + 1)
    };

    let maturity_cycle = state
        .cycle
        .checked_add(world.constants.min_maturity_lead)
        .and_then(|value| value.checked_add(offset))
        .ok_or(Guard::CycleOverflow)?;

    AnnounceMaturity {
        maturity_cycle,
        signers: std::iter::once(OPERATOR_KEY).collect(),
        fee_envelope: FeeEnvelope::default(),
    }
    .apply(world, next_model_order(world)?)
}

// ´rule:verification:property-open-injection´
//
// The injection action is derived from the current world: the decoy
// RESV uses the current active reserve value, the entitlement target
// uses the current pending cycle, and malformed requests use the
// current pool id. This is exactly why materialization must happen
// inside the evolving-world loop.

fn property_open_injection(world: &World, seed: PropertyActionSeed) -> PropertyAction {
    match seed.shape_selector % 6 {
        0 => {
            // Malformed request for this pool: principal equals gross
            // value. It must remain inert and fail only at attempted
            // admission.
            let value = projected_positive(seed.amount_a, PROPERTY_PRINCIPAL_MAX)
                .expect("bounded malformed-request value");

            PropertyAction::InjectOpen {
                asset: Asset::Lbtc,
                value,
                meta: Meta::DepositRequest {
                    pool_id: world.constants.pool_id,
                    refund_key: seed.owner(),
                    receipt_owner: seed.other_owner(),
                    deposit_principal: value,
                },
            }
        }

        1 => {
            // Well-formed request for another pool.
            let principal =
                projected_principal(seed.amount_a).expect("bounded cross-pool principal");

            let budget = projected_budget(seed.amount_b).expect("bounded cross-pool budget");

            let gross = principal
                .checked_add(budget)
                .expect("bounded cross-pool gross");

            PropertyAction::InjectOpen {
                asset: Asset::Lbtc,
                value: gross,
                meta: Meta::DepositRequest {
                    pool_id: world.constants.pool_id.wrapping_add(1),
                    refund_key: seed.owner(),
                    receipt_owner: seed.other_owner(),
                    deposit_principal: principal,
                },
            }
        }

        2 => {
            // Decoy RESV at the active reserve's current value.
            let value = world
                .active_resv()
                .map(|(_, reserve)| reserve.value)
                .unwrap_or(Sat::ZERO);

            PropertyAction::InjectOpen {
                asset: Asset::Lbtc,
                value,
                meta: Meta::Resv,
            }
        }

        3 => {
            // Zero-value anchor-shaped junk.
            PropertyAction::InjectOpen {
                asset: Asset::Lbtc,
                value: Sat::ZERO,
                meta: Meta::CpfpAnchor,
            }
        }

        4 => {
            // Foreign asset carrying a state-shaped payload.
            let fake_state = PoolState {
                omega: Sat::ZERO,
                y_l: Sat::ZERO,
                y_t: Sat::ZERO,
                q: Sat::ZERO,
                cycle: seed.amount_a,
                maturity: Maturity::Complete,
            };

            PropertyAction::InjectOpen {
                asset: Asset::Foreign(PROPERTY_FOREIGN_ASSET_ID),
                value: Sat::ONE,
                meta: Meta::State(fake_state),
            }
        }

        _ => {
            // Foreign asset carrying an entitlement-shaped payload.
            PropertyAction::InjectOpen {
                asset: Asset::Foreign(PROPERTY_FOREIGN_ASSET_ID),
                value: projected_positive(seed.amount_a, 1_024)
                    .expect("bounded foreign entitlement value"),
                meta: Meta::DepositEntitlement {
                    owner: seed.owner(),
                    target_cycle: world
                        .state()
                        .ok()
                        .and_then(|(_, state)| state.next_cycle().ok())
                        .unwrap_or(0),
                },
            }
        }
    }
}

// ´def:verification:property-action´

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyAction {
    InjectOpen {
        asset: Asset,
        value: Sat,
        meta: Meta,
    },

    AdvanceBlocks {
        blocks: u64,
    },

    CreateRequest {
        funder: OwnerKey,
        owner: OwnerKey,
        principal: Sat,
        budget: Sat,
    },

    CancelSomeRequest,

    AdmitSomeRequests,

    RunCycle {
        caller: CycleCaller,
    },

    SettleSomeEntitlements,

    TransferSomeLive,

    TransferSomeTimeLocked,

    RedeemSomeLive,

    RelabelSomeTimeLocked,

    BurnSomeLive {
        owner_selector: u8,
        burn_selector: u64,
        record_selector: u64,
        address_selector: u8,
    },

    CompactSomeAsh,

    ClearSomeAsh,

    AnnounceAtMinimumLead,
}

// ´def:verification:property-step-result´

// The spec-declared shape carries the successor `World` by value; the
// size imbalance against `Rejected` is accepted for fidelity.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyStepResult {
    Applied(World),
    Rejected(Guard),
}

// ´rule:verification:property-step´

pub fn apply_property_action(world: &World, action: PropertyAction) -> PropertyStepResult {
    let before = world.clone();

    let result = match action {
        PropertyAction::InjectOpen { asset, value, meta } => {
            inject_open_object(world, asset, value, meta)
        }

        PropertyAction::AdvanceBlocks { blocks } => {
            let mut next = world.clone();

            match next.pace_age_blocks.checked_add(blocks) {
                Some(value) => {
                    next.pace_age_blocks = value;
                    Ok(next)
                }

                None => Err(Guard::Overflow),
            }
        }

        PropertyAction::CreateRequest {
            funder,
            owner,
            principal,
            budget,
        } => property_create_request(world, funder, owner, principal, budget),

        PropertyAction::CancelSomeRequest => property_cancel_request(world),

        PropertyAction::AdmitSomeRequests => property_admit_requests(world),

        PropertyAction::RunCycle { caller } => property_run_cycle(world, caller),

        PropertyAction::SettleSomeEntitlements => property_settle_distribution(world),

        PropertyAction::TransferSomeLive => {
            property_transfer(world, ReceiptClass::Live, PROPERTY_OWNER_B)
        }

        PropertyAction::TransferSomeTimeLocked => {
            property_transfer(world, ReceiptClass::TimeLocked, PROPERTY_OWNER_C)
        }

        PropertyAction::RedeemSomeLive => property_redeem(world),

        PropertyAction::RelabelSomeTimeLocked => property_relabel(world),

        PropertyAction::BurnSomeLive {
            owner_selector,
            burn_selector,
            record_selector,
            address_selector,
        } => property_burn(
            world,
            owner_selector,
            burn_selector,
            record_selector,
            address_selector,
        ),

        PropertyAction::CompactSomeAsh => property_compact_ash(world),

        PropertyAction::ClearSomeAsh => property_clear(world),

        PropertyAction::AnnounceAtMinimumLead => property_announce_maturity(
            world,
            PropertyActionSeed {
                selector: 0,
                amount_a: 0,
                amount_b: 0,
                owner_selector: 0,
                shape_selector: 0,
            },
        ),
    };

    match result {
        Ok(next) => {
            assert!(
                check_invariant(&next).is_ok(),
                "accepted transition broke invariant",
            );

            // L-floor, asserted directly in the goal register,
            // independent of branch arithmetic: Ω'·Y ≥ Ω·Y' by u128
            // cross-multiplication for every accepted transition.
            if let (Ok((_, old_state)), Ok((_, new_state))) = (before.state(), next.state()) {
                let old_y = old_state.y().expect("valid predecessor supply");
                let new_y = new_state.y().expect("valid successor supply");

                assert!(
                    u128::from(new_state.omega.get()) * u128::from(old_y.get())
                        >= u128::from(old_state.omega.get()) * u128::from(new_y.get()),
                    "L-floor violated by accepted transition",
                );
            }

            // L-record: the burn record is append-only across every
            // accepted transition.
            assert!(
                next.history
                    .transitions
                    .starts_with(&before.history.transitions),
                "accepted transition rewrote history",
            );

            PropertyStepResult::Applied(next)
        }

        Err(error) => {
            assert_eq!(world, &before, "rejected transition mutated world");

            PropertyStepResult::Rejected(error)
        }
    }
}

// ´rule:verification:materialize-action´

/// Every action family is reachable: the fifteen selector values map
/// onto block advancement, all thirteen operation families, and
/// state-aware adversarial open-asset injection.
pub fn materialize_action(world: &World, seed: PropertyActionSeed) -> PropertyAction {
    match seed.selector % 15 {
        0 => PropertyAction::AdvanceBlocks {
            blocks: 1 + (seed.amount_a % 256),
        },

        1 => PropertyAction::CreateRequest {
            funder: seed.owner(),
            owner: seed.other_owner(),

            principal: projected_principal(seed.amount_a).expect("bounded property principal"),

            budget: projected_budget(seed.amount_b).expect("bounded property budget"),
        },

        2 => PropertyAction::CancelSomeRequest,

        3 => PropertyAction::AdmitSomeRequests,

        4 => PropertyAction::RunCycle {
            caller: if seed.amount_a.is_multiple_of(2) {
                CycleCaller::Anyone
            } else {
                CycleCaller::Operator
            },
        },

        5 => PropertyAction::SettleSomeEntitlements,

        6 => PropertyAction::TransferSomeLive,

        7 => PropertyAction::TransferSomeTimeLocked,

        8 => PropertyAction::RedeemSomeLive,

        9 => PropertyAction::RelabelSomeTimeLocked,

        10 => PropertyAction::BurnSomeLive {
            owner_selector: seed.owner_selector,
            burn_selector: seed.amount_a,
            record_selector: seed.amount_b,
            address_selector: seed.shape_selector,
        },

        11 => PropertyAction::CompactSomeAsh,

        12 => PropertyAction::ClearSomeAsh,

        13 => PropertyAction::AnnounceAtMinimumLead,

        _ => property_open_injection(world, seed),
    }
}

// ´rule:verification:property-seed-trace´
//
// Each seed is materialized against the evolving world, so state-aware
// injections (decoy RESV value, pending entitlement cycle) track the
// trace as it progresses.

pub fn drive_property_seed_trace(
    initial: &World,
    seeds: impl IntoIterator<Item = PropertyActionSeed>,
) -> World {
    let mut world = initial.clone();

    for seed in seeds {
        let action = materialize_action(&world, seed);

        match apply_property_action(&world, action) {
            PropertyStepResult::Applied(next) => {
                world = next;
            }

            PropertyStepResult::Rejected(_) => {}
        }
    }

    world
}

// ´rule:verification:property-trace´

pub fn drive_property_trace(
    initial: &World,
    actions: impl IntoIterator<Item = PropertyAction>,
) -> World {
    let mut world = initial.clone();

    for action in actions {
        match apply_property_action(&world, action) {
            PropertyStepResult::Applied(next) => {
                world = next;
            }

            PropertyStepResult::Rejected(_) => {}
        }
    }

    world
}

/// Stable coverage name for each property-action family.
pub fn property_action_name(action: &PropertyAction) -> &'static str {
    match action {
        PropertyAction::InjectOpen { .. } => "inject-open",
        PropertyAction::AdvanceBlocks { .. } => "advance-blocks",
        PropertyAction::CreateRequest { .. } => "create-request",
        PropertyAction::CancelSomeRequest => "cancel-request",
        PropertyAction::AdmitSomeRequests => "admit-deposits",
        PropertyAction::RunCycle { .. } => "cycle",
        PropertyAction::SettleSomeEntitlements => "settle-distribution",
        PropertyAction::TransferSomeLive => "transfer-live",
        PropertyAction::TransferSomeTimeLocked => "transfer-time-locked",
        PropertyAction::RedeemSomeLive => "redeem",
        PropertyAction::RelabelSomeTimeLocked => "receipt-relabel",
        PropertyAction::BurnSomeLive { .. } => "burn",
        PropertyAction::CompactSomeAsh => "compact-ash",
        PropertyAction::ClearSomeAsh => "clear",
        PropertyAction::AnnounceAtMinimumLead => "announce-maturity",
    }
}
