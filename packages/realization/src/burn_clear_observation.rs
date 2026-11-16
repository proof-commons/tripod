//! Internal primitive carriers bound to a normalized observation snapshot.
//!
//! Context identifiers are equality tokens supplied by an independent producer,
//! not signatures or public realization identities. Binding checks cannot prove
//! that a producer truthfully observed a transaction or STATE metadata.

use architecture::{AssetId, BoundId, ObjectId, OperationId, ProjectionId, TagId};

use crate::{
    BurnAddress, BurnClearRefusal, ObservedAsset, ObservedObjectKind, ObservedObjectRef,
    ObservedSide, OperationObservation, ProtocolAmount, RealizationError, StateMetadata,
};

/// Exact context shared by independently offered projection subjects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurnClearContext {
    pub(crate) common: OperationObservation,
    transaction: [u8; 32],
    certificate: [u8; 32],
}

impl BurnClearContext {
    /// Bind producer-supplied identity tokens to a normalized common snapshot.
    /// The producer authenticates the tokens; this crate checks exact equality.
    pub fn bind_observation(
        common: OperationObservation,
        transaction: [u8; 32],
        certificate: [u8; 32],
    ) -> Result<Self, RealizationError> {
        Ok(Self {
            common: common.validate_and_normalize()?,
            transaction,
            certificate,
        })
    }

    /// Read the immutable normalized snapshot, including exact region members.
    pub fn read_observation(&self) -> &OperationObservation {
        &self.common
    }
}

/// A primitive STATE payload attached to one common object reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedState {
    /// Exact selected-region reference, including its side.
    pub reference: ObservedObjectRef,
    /// Independently observed semantic metadata, never decoded here.
    pub metadata: StateMetadata,
}

/// One Burn data output; its position is semantic, not a transaction index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BurnRecord {
    /// Contiguous zero-based position within this transaction's record family.
    pub ordinal: u32,
    /// A 32-byte realization-domain address; repetitions are permitted.
    pub address: BurnAddress,
    /// Positive scalar claim; aggregate credit is a separate decision.
    pub amount: ProtocolAmount,
    /// Must be the architecture's Burn tag.
    pub tag: TagId,
    /// The fresh ASH this output names, in the enclosing context.
    pub fresh_ash: ObservedObjectRef,
}

/// One typed destruction data output, distinct from a canonical flow leg.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Destruction {
    /// Architecture asset identity.
    pub asset: AssetId,
    /// Public destroyed quantity.
    pub amount: ProtocolAmount,
    /// Architecture destruction tag.
    pub tag: TagId,
    /// Position of the associated flow in the normalized partition.
    pub flow: Option<usize>,
}

/// Operation-specific primitive facts. Compact has no event or data payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BurnClearPayload {
    /// Fresh ASH and the ordered record family of one Burn.
    Burn {
        /// Unique fresh output reference.
        fresh_ash: ObservedObjectRef,
        /// Zero through the common observation's `BurnRecordMax` records.
        records: Vec<BurnRecord>,
    },
    /// STATE pair and data output of one Clear.
    Clear {
        /// Exact predecessor carrier.
        predecessor: ObservedState,
        /// Exact successor carrier.
        successor: ObservedState,
        /// Exact source family, in common reference order.
        ash_sources: Vec<ObservedObjectRef>,
        /// One separately associated data output.
        destruction: Destruction,
    },
    /// A compact observation with no invented Burn/Clear payload.
    Compact,
}

/// Unchecked extension; constructibility does not confer projection validity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurnClearObservation {
    /// Snapshot and producer identity tokens to compare with the expected context.
    pub context: BurnClearContext,
    /// Primitive operation-specific fields.
    pub payload: BurnClearPayload,
}

/// An extension whose references and structure are bound, not an accepted
/// operation. Declarations, independent model conformance and authentication
/// retain their own acceptance boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundBurnClearObservation {
    pub(crate) observation: BurnClearObservation,
}

impl BoundBurnClearObservation {
    /// Read the checked primitive extension without permitting mutation.
    pub fn read_observation(&self) -> &BurnClearObservation {
        &self.observation
    }
}

/// Validate structure and exact context independently of aggregate credit.
/// Missing extension never supplies default STATE fields or projection facts.
pub fn validate_burn_clear_observation(
    common: &OperationObservation,
    expected: &BurnClearContext,
    extension: Option<&BurnClearObservation>,
) -> Result<BoundBurnClearObservation, BurnClearRefusal> {
    let span = tracing::debug_span!(target: "realization::burn_clear_observation",
        "validate_burn_clear_observation", operation_code = common.operation.code());
    let _guard = span.enter();
    let result = validate_core(common, expected, extension);
    match &result {
        Ok(_) => tracing::debug!(target: "realization::burn_clear_observation",
            outcome = "completed", "burn clear observation validated"),
        Err(refusal) => tracing::debug!(target: "realization::burn_clear_observation",
            outcome = "refused", refusal_kind = refusal.name(),
            "burn clear observation validated"),
    }
    result
}

fn validate_core(
    common: &OperationObservation,
    expected: &BurnClearContext,
    extension: Option<&BurnClearObservation>,
) -> Result<BoundBurnClearObservation, BurnClearRefusal> {
    use BurnClearRefusal as R;
    let extension = extension.ok_or(R::MissingExtension)?;
    let normalized = common
        .clone()
        .validate_and_normalize_core()
        .map_err(|_| R::MalformedObservation)?;
    if normalized != expected.common || extension.context.common != normalized {
        return Err(R::CommonObservation);
    }
    if extension.context.transaction != expected.transaction {
        return Err(R::Transaction);
    }
    if extension.context.certificate != expected.certificate
        || !normalized
            .projections
            .contains(&ProjectionId::TransitionCertificate)
    {
        return Err(R::Certificate);
    }
    validate_payload(&normalized, &extension.payload)?;
    Ok(BoundBurnClearObservation {
        observation: extension.clone(),
    })
}

fn validate_payload(
    normalized: &OperationObservation,
    payload: &BurnClearPayload,
) -> Result<(), BurnClearRefusal> {
    use BurnClearRefusal as R;
    match payload {
        BurnClearPayload::Burn { fresh_ash, records } => {
            if normalized.operation != OperationId::Burn {
                return Err(R::Operation);
            }
            if family(normalized, ObservedSide::Output, ObjectId::Ash) != [*fresh_ash] {
                return Err(R::AshBinding);
            }
            read_amount(normalized, *fresh_ash, ObjectId::Ash)?;
            let maximum = normalized
                .bounds
                .get(&BoundId::BurnRecordMax)
                .ok_or(R::RecordBound)?;
            if u64::try_from(records.len()).map_err(|_| R::RecordBound)? > maximum.get() {
                return Err(R::RecordBound);
            }
            for (ordinal, record) in records.iter().enumerate() {
                if record.amount.is_zero() || record.tag != TagId::Burn {
                    return Err(R::RecordStructure);
                }
                if usize::try_from(record.ordinal).ok() != Some(ordinal) {
                    return Err(R::RecordOrdinal);
                }
                if record.fresh_ash != *fresh_ash {
                    return Err(R::DataAssociation);
                }
            }
        }
        BurnClearPayload::Clear {
            predecessor,
            successor,
            ash_sources,
            destruction,
        } => {
            if normalized.operation != OperationId::Clear {
                return Err(R::Operation);
            }
            for (state, side) in [
                (predecessor, ObservedSide::Input),
                (successor, ObservedSide::Output),
            ] {
                if family(normalized, side, ObjectId::State) != [state.reference] {
                    return Err(R::StateBinding);
                }
                if !normalized.object(state.reference).is_some_and(|object| {
                    object.asset == ObservedAsset::Declared(AssetId::Pid)
                        && object.value.is(ProtocolAmount::ONE)
                }) {
                    return Err(R::StateBinding);
                }
            }
            if ash_sources.is_empty()
                || *ash_sources != family(normalized, ObservedSide::Input, ObjectId::Ash)
            {
                return Err(R::AshBinding);
            }
            for source in ash_sources {
                read_amount(normalized, *source, ObjectId::Ash)?;
            }
            let flow = destruction
                .flow
                .and_then(|index| normalized.region.canonical_partition.flows.get(index))
                .ok_or(R::DataAssociation)?;
            if flow.sources != *ash_sources {
                return Err(R::DataAssociation);
            }
            if destruction.asset != AssetId::U
                || destruction.tag != TagId::Recon
                || destruction.amount.is_zero()
                || flow.asset != destruction.asset
                || flow.destructions.len() != 1
                || flow.destructions[0].tag != destruction.tag
                || flow.destructions[0].amount != destruction.amount
            {
                return Err(R::DestructionBinding);
            }
            let partition = &normalized.region.canonical_partition;
            if !partition.issuances.is_empty()
                || partition
                    .flows
                    .iter()
                    .flat_map(|flow| &flow.destructions)
                    .take(2)
                    .count()
                    != 1
            {
                return Err(R::DestructionBinding);
            }
        }
        BurnClearPayload::Compact => {
            if normalized.operation != OperationId::CompactAsh {
                return Err(R::Operation);
            }
        }
    }
    Ok(())
}

pub(crate) fn family(
    common: &OperationObservation,
    side: ObservedSide,
    kind: ObjectId,
) -> Vec<ObservedObjectRef> {
    common
        .declared_objects(side, kind)
        .map(|object| object.reference)
        .collect()
}

pub(crate) fn read_amount(
    common: &OperationObservation,
    reference: ObservedObjectRef,
    kind: ObjectId,
) -> Result<ProtocolAmount, BurnClearRefusal> {
    let object = common
        .object(reference)
        .ok_or(BurnClearRefusal::AshBinding)?;
    if object.kind != ObservedObjectKind::Declared(kind)
        || object.asset != ObservedAsset::Declared(AssetId::U)
    {
        return Err(BurnClearRefusal::AshBinding);
    }
    object
        .value
        .protocol()
        .filter(|amount| !amount.is_zero())
        .ok_or(BurnClearRefusal::AshBinding)
}
