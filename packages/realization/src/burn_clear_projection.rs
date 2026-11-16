//! Separate internal projection subjects, not report producers or declaration
//! conformance. No successful role substitutes for another role's verdict.

use architecture::{BoundId, ObjectId, OperationId, ProjectionId, RootId};

use crate::burn_clear_observation::{family, read_amount};
use crate::{
    BoundBurnClearObservation, BurnClearContext, BurnClearPayload, BurnClearRefusal, BurnRecord,
    Destruction, FactId, ObservedObjectRef, ObservedRootEffectKind, ObservedSide, ObservedState,
    ProtocolAmount, SemanticValue, StateField, TransactionSide,
};

/// Burn's event subject. Context includes live sources/change and all owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurnEventProjection {
    /// Exact transaction, certificate and normalized common snapshot.
    pub context: BurnClearContext,
    /// Unique fresh ASH reference.
    pub fresh_ash: ObservedObjectRef,
    /// Independently offered fresh amount.
    pub fresh_amount: ProtocolAmount,
    /// Canonical ordered records, including their data-output associations.
    pub records: Vec<BurnRecord>,
}

/// Whole-family credit verdict. Overclaim leaves structure and Burn untouched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordCredit {
    /// Every structurally valid claim is credited, including the empty family.
    Credited {
        /// Exact aggregate, intentionally wider than `ProtocolAmount`.
        total: u128,
    },
    /// No record in the family receives credit.
    Overclaim {
        /// Exact aggregate, including values beyond the scalar amount domain.
        total: u128,
        /// The bound fresh amount, never a compact or residual amount.
        fresh_amount: ProtocolAmount,
    },
}

/// Clear's event payload plus exact contextual associations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClearEventProjection {
    /// Exact context, not merely matching payload quantities.
    pub context: BurnClearContext,
    /// Exact predecessor carrier.
    pub predecessor: ObservedState,
    /// Exact successor carrier.
    pub successor: ObservedState,
    /// Exact ASH input references.
    pub ash_sources: Vec<ObservedObjectRef>,
    /// B, independently offered for comparison with the source family.
    pub aggregate: ProtocolAmount,
    /// X, recomputed from the bound predecessor and ASH family by `clear_ash`.
    pub clear_amount: ProtocolAmount,
    /// The associated U/Recon data output.
    pub destruction: Destruction,
    /// Actual residual output references and quantities; absence is an empty list.
    pub residual: Vec<(ObservedObjectRef, ProtocolAmount)>,
    /// Event omega from the actual successor.
    pub omega: ProtocolAmount,
    /// Derived successor supply, never a stored field.
    pub y: ProtocolAmount,
}

/// Independently checked STATE-edge subject; success is not event acceptance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateSuccessionProjection {
    /// Exact transaction and certificate context.
    pub context: BurnClearContext,
    /// Expected predecessor reference and six fields.
    pub predecessor: ObservedState,
    /// Expected successor reference and six fields.
    pub successor: ObservedState,
}

/// Full scoped accounting vocabulary, including Burn and compact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountingValues {
    /// Burn record totals and fresh value; a total is not a credit verdict.
    Burn {
        /// Wide aggregate of all record claims.
        record_total: u128,
        /// Fresh ASH value.
        fresh_amount: ProtocolAmount,
    },
    /// Compact value preservation, with both sides independently observed.
    Compact {
        /// Input ASH aggregate.
        input: ProtocolAmount,
        /// Output ASH aggregate.
        output: ProtocolAmount,
    },
    /// Every quantity in ruling 147's Clear row.
    Clear {
        /// Input aggregate B.
        aggregate: ProtocolAmount,
        /// Recomputed clamp X.
        clear_amount: ProtocolAmount,
        /// Actual predecessor minus successor `y_l`.
        state_decrement: ProtocolAmount,
        /// Actual associated destruction, including tag and asset.
        destruction: Destruction,
        /// Actual residual aggregate.
        residual: ProtocolAmount,
    },
}

/// Accounting is a separate subject from either event, credit or STATE edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountingProjection {
    /// Exact context; foreign regions cannot contribute operands.
    pub context: BurnClearContext,
    /// Operation-specific complete quantity list.
    pub values: AccountingValues,
    /// Event type; compact has no event.
    pub event: Option<ProjectionId>,
    /// Observed residue projection presence. Validation requires absence here;
    /// this scoped claim establishes no later-history noninterference theorem.
    pub distribution_residue: bool,
}

/// Compact silence is separately bound rather than inferred from accounting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactSilenceProjection {
    /// Exact compact context whose absence claims are checked.
    pub context: BurnClearContext,
}

impl BoundBurnClearObservation {
    /// Resolve only a carried Clear STATE fact. Missing or foreign-operation
    /// facts stay absent; the legacy evaluator is deliberately unchanged.
    pub fn resolve_state_fact(&self, fact: &FactId) -> Option<SemanticValue> {
        let BurnClearPayload::Clear {
            predecessor,
            successor,
            ..
        } = &self.observation.payload
        else {
            return None;
        };
        let FactId::StateField {
            operation: OperationId::Clear,
            side,
            field,
        } = fact
        else {
            return None;
        };
        let metadata = match side {
            TransactionSide::Input => &predecessor.metadata,
            TransactionSide::Output => &successor.metadata,
        };
        Some(match field {
            StateField::Omega => SemanticValue::Amount(metadata.omega),
            StateField::YL => SemanticValue::Amount(metadata.y_l),
            StateField::YT => SemanticValue::Amount(metadata.y_t),
            StateField::Q => SemanticValue::Amount(metadata.q),
            StateField::Cycle => SemanticValue::Cycle(metadata.cycle),
            StateField::Maturity => SemanticValue::Maturity(metadata.maturity),
        })
    }
}

/// Validate an independently offered Burn event. Record overclaim is not read.
pub fn validate_burn_event(
    bound: &BoundBurnClearObservation,
    offered: &BurnEventProjection,
) -> Result<(), BurnClearRefusal> {
    let BurnClearPayload::Burn { fresh_ash, records } = &bound.observation.payload else {
        return Err(BurnClearRefusal::ProjectionRole);
    };
    let context = &bound.observation.context;
    let common = &context.common;
    let fresh_amount = read_amount(common, *fresh_ash, ObjectId::Ash)?;
    if offered.context != *context
        || offered.fresh_ash != *fresh_ash
        || offered.fresh_amount != fresh_amount
        || offered.records != *records
        || !has_burn_classes(common, *fresh_ash)
        || !common.projections.contains(&ProjectionId::BurnEvent)
        || common.projections.contains(&ProjectionId::ClearEvent)
        || common
            .projections
            .contains(&ProjectionId::DistributionResidue)
        || !common.root_effects.is_empty()
        || common
            .region_objects()
            .any(|object| object.kind == crate::ObservedObjectKind::Declared(ObjectId::State))
        || !common.region.canonical_partition.issuances.is_empty()
        || common
            .region
            .canonical_partition
            .flows
            .iter()
            .any(|flow| !flow.destructions.is_empty())
    {
        return Err(BurnClearRefusal::BurnEvent);
    }
    Ok(())
}

/// Decide whole-family record credit after structural binding.
///
/// A Vec's maximum
/// length times a 51-bit scalar fits u128 on supported 32/64-bit hosts; checked
/// arithmetic additionally preserves totality on any future wider host.
pub fn evaluate_record_credit(
    bound: &BoundBurnClearObservation,
) -> Result<RecordCredit, BurnClearRefusal> {
    let BurnClearPayload::Burn { fresh_ash, records } = &bound.observation.payload else {
        return Err(BurnClearRefusal::ProjectionRole);
    };
    let total = record_total(records)?;
    let fresh_amount = read_amount(&bound.observation.context.common, *fresh_ash, ObjectId::Ash)?;
    Ok(if total <= u128::from(fresh_amount.get()) {
        RecordCredit::Credited { total }
    } else {
        RecordCredit::Overclaim {
            total,
            fresh_amount,
        }
    })
}

/// Validate Clear's event payload and contextual associations, separately from
/// the assignment and accounting validators.
pub fn validate_clear_event(
    bound: &BoundBurnClearObservation,
    offered: &ClearEventProjection,
) -> Result<(), BurnClearRefusal> {
    let BurnClearPayload::Clear {
        predecessor,
        successor,
        ash_sources,
        destruction,
    } = &bound.observation.payload
    else {
        return Err(BurnClearRefusal::ProjectionRole);
    };
    let context = &bound.observation.context;
    let common = &context.common;
    let residual = ash_values(bound, ObservedSide::Output)?;
    let clear_amount = clear_transition(bound)?.clear_amount;
    let aggregate = sum_ash(bound, ObservedSide::Input)?;
    let y = successor
        .metadata
        .y_l
        .checked_add(successor.metadata.y_t)
        .map_err(|_| BurnClearRefusal::Arithmetic)?;
    if offered.context != *context
        || offered.predecessor != *predecessor
        || offered.successor != *successor
        || offered.ash_sources != *ash_sources
        || offered.aggregate != aggregate
        || offered.clear_amount != clear_amount
        || offered.destruction != *destruction
        || offered.residual != residual
        || offered.omega != successor.metadata.omega
        || offered.y != y
        || destruction.amount != clear_amount
        || common
            .root_effects
            .iter()
            .any(|effect| effect.root != RootId::State)
        || !common.projections.contains(&ProjectionId::ClearEvent)
        || common.projections.contains(&ProjectionId::BurnEvent)
        || common
            .projections
            .contains(&ProjectionId::DistributionResidue)
    {
        return Err(BurnClearRefusal::ClearEvent);
    }
    Ok(())
}

/// Validate the exact STATE edge and the six-field successor using Clear's
/// already-owned transition. This is not a new declaration or report producer.
pub fn validate_state_succession(
    bound: &BoundBurnClearObservation,
    offered: &StateSuccessionProjection,
) -> Result<(), BurnClearRefusal> {
    let BurnClearPayload::Clear {
        predecessor,
        successor,
        ..
    } = &bound.observation.payload
    else {
        return Err(BurnClearRefusal::ProjectionRole);
    };
    let context = &bound.observation.context;
    let effects = &context.common.root_effects;
    if offered.context != *context
        || offered.predecessor != *predecessor
        || offered.successor != *successor
        || effects.len() != 1
        || effects[0].root != RootId::State
        || effects[0].effect != ObservedRootEffectKind::Succession
        || clear_transition(bound)?.successor != successor.metadata
    {
        return Err(BurnClearRefusal::StateSuccession);
    }
    Ok(())
}

/// Validate the full operation-scoped accounting list independently of events.
///
/// Residue presence is refused; statements about later operations require the
/// separately bound composition subjects assigned to bite 2.8.
pub fn validate_accounting(
    bound: &BoundBurnClearObservation,
    offered: &AccountingProjection,
) -> Result<(), BurnClearRefusal> {
    let context = &bound.observation.context;
    let (values, event) = match &bound.observation.payload {
        BurnClearPayload::Burn { fresh_ash, records } => (
            AccountingValues::Burn {
                record_total: record_total(records)?,
                fresh_amount: read_amount(&context.common, *fresh_ash, ObjectId::Ash)?,
            },
            Some(ProjectionId::BurnEvent),
        ),
        BurnClearPayload::Compact => {
            let input = sum_ash(bound, ObservedSide::Input)?;
            let output = sum_ash(bound, ObservedSide::Output)?;
            if input != output {
                return Err(BurnClearRefusal::Accounting);
            }
            (AccountingValues::Compact { input, output }, None)
        }
        BurnClearPayload::Clear {
            predecessor,
            successor,
            destruction,
            ..
        } => {
            let transition = clear_transition(bound)?;
            let state_decrement = predecessor
                .metadata
                .y_l
                .checked_sub(successor.metadata.y_l)
                .map_err(|_| BurnClearRefusal::Arithmetic)?;
            let residual = sum_ash(bound, ObservedSide::Output)?;
            if state_decrement != transition.clear_amount
                || destruction.amount != transition.clear_amount
                || residual != transition.residual
            {
                return Err(BurnClearRefusal::Accounting);
            }
            (
                AccountingValues::Clear {
                    aggregate: sum_ash(bound, ObservedSide::Input)?,
                    clear_amount: transition.clear_amount,
                    state_decrement,
                    destruction: *destruction,
                    residual,
                },
                Some(ProjectionId::ClearEvent),
            )
        }
    };
    let actual_events: Vec<_> = context
        .common
        .projections
        .iter()
        .filter(|projection| {
            matches!(
                projection,
                ProjectionId::BurnEvent | ProjectionId::ClearEvent
            )
        })
        .copied()
        .collect();
    if offered.context != *context
        || offered.values != values
        || offered.event != event
        || offered.distribution_residue
        || actual_events != event.into_iter().collect::<Vec<_>>()
        || context
            .common
            .projections
            .contains(&ProjectionId::DistributionResidue)
    {
        return Err(BurnClearRefusal::Accounting);
    }
    Ok(())
}

/// Validate compact's lack of Burn/Clear provenance and monetary side effects.
pub fn validate_compact_silence(
    bound: &BoundBurnClearObservation,
    offered: &CompactSilenceProjection,
) -> Result<(), BurnClearRefusal> {
    if !matches!(bound.observation.payload, BurnClearPayload::Compact) {
        return Err(BurnClearRefusal::ProjectionRole);
    }
    let context = &bound.observation.context;
    let common = &context.common;
    if offered.context != *context
        || !common.root_effects.is_empty()
        || common.projections.iter().any(|projection| {
            matches!(
                projection,
                ProjectionId::BurnEvent
                    | ProjectionId::ClearEvent
                    | ProjectionId::DistributionResidue
            )
        })
        || common
            .region_objects()
            .any(|object| object.kind == crate::ObservedObjectKind::Declared(ObjectId::State))
        || !common.region.canonical_partition.issuances.is_empty()
        || common
            .region
            .canonical_partition
            .flows
            .iter()
            .any(|flow| !flow.destructions.is_empty())
    {
        return Err(BurnClearRefusal::CompactSilence);
    }
    Ok(())
}

fn record_total(records: &[BurnRecord]) -> Result<u128, BurnClearRefusal> {
    records.iter().try_fold(0_u128, |total, record| {
        total
            .checked_add(u128::from(record.amount.get()))
            .ok_or(BurnClearRefusal::Arithmetic)
    })
}

fn ash_values(
    bound: &BoundBurnClearObservation,
    side: ObservedSide,
) -> Result<Vec<(ObservedObjectRef, ProtocolAmount)>, BurnClearRefusal> {
    let common = &bound.observation.context.common;
    family(common, side, ObjectId::Ash)
        .into_iter()
        .map(|reference| Ok((reference, read_amount(common, reference, ObjectId::Ash)?)))
        .collect()
}

fn sum_ash(
    bound: &BoundBurnClearObservation,
    side: ObservedSide,
) -> Result<ProtocolAmount, BurnClearRefusal> {
    ProtocolAmount::checked_sum(
        ash_values(bound, side)?
            .into_iter()
            .map(|(_, amount)| amount),
    )
    .map_err(|_| BurnClearRefusal::Arithmetic)
}

fn clear_transition(
    bound: &BoundBurnClearObservation,
) -> Result<crate::ClearTransitionResult, BurnClearRefusal> {
    let BurnClearPayload::Clear { predecessor, .. } = &bound.observation.payload else {
        return Err(BurnClearRefusal::ProjectionRole);
    };
    let maximum = bound
        .observation
        .context
        .common
        .bounds
        .get(&BoundId::AshBatchMax)
        .and_then(|count| usize::try_from(count.get()).ok())
        .ok_or(BurnClearRefusal::ClearTransition)?;
    crate::clear_ash(
        &predecessor.metadata,
        &ash_values(bound, ObservedSide::Input)?,
        maximum,
    )
    .map_err(|refusal| match refusal {
        crate::ClearTransitionRefusal::SupplyOutOfDomain
        | crate::ClearTransitionRefusal::AshTotalOutOfDomain => BurnClearRefusal::Arithmetic,
        _ => BurnClearRefusal::ClearTransition,
    })
}

fn has_burn_classes(common: &crate::OperationObservation, fresh_ash: ObservedObjectRef) -> bool {
    common
        .declared_objects(ObservedSide::Input, ObjectId::ReceiptLive)
        .any(|object| object.asset == crate::ObservedAsset::Declared(architecture::AssetId::U))
        && !common.region_objects().any(|object| {
            object.asset == crate::ObservedAsset::Declared(architecture::AssetId::U)
                && object.reference != fresh_ash
                && object.kind != crate::ObservedObjectKind::Declared(ObjectId::ReceiptLive)
        })
}
