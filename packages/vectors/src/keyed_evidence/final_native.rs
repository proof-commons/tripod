//! Exact-subject native joins beside, never substituted for, offline validators.
use crate::keyed_final_capture::{CaptureBoundary, CaptureRow, CaptureSchema};
use serde_json::Value;
use target_elements_conformance::constructor::tagged::sha256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JoinRefusal {
    Subject,
    Layer,
    Route,
    Reason,
    Block,
    Readback,
    Weight,
    Dimension,
    Policy,
    Structural,
}

/// A reading of this one final subject; it grants no reusable policy guarantee.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinalBytePolicyObservation {
    pub subject: [u8; 32],
    pub route: String,
    pub relay_allowed: bool,
    pub structural: bool,
}

/// Joint discharge exists only for the same bytes in a freshly prepared reader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberExecutionAndRelationEvidence {
    pub subject: [u8; 32],
    pub event: crate::keyed_history_reader::EventId,
    pub relations: Vec<crate::keyed_history_reader::RelationEvidence>,
    pub policy: FinalBytePolicyObservation,
}

pub struct NativeHistory {
    input: crate::keyed_history_reader::Input,
    reading: crate::keyed_history_reader::Reading,
}
impl NativeHistory {
    pub(crate) fn prepare(
        input: crate::keyed_history_reader::Input,
    ) -> Result<Self, crate::keyed_history_reader::Refusal> {
        let reading = crate::keyed_history_reader::read(&input)?;
        Ok(Self { input, reading })
    }
    pub(crate) const fn reading(&self) -> &crate::keyed_history_reader::Reading {
        &self.reading
    }
    pub(crate) fn join(
        &self,
        event: u32,
        row: &CaptureRow,
        observation: &Value,
        diagnostic: &str,
    ) -> Result<MemberExecutionAndRelationEvidence, JoinRefusal> {
        if self.input.bytes.get(&event) != Some(&row.submitted) {
            return Err(JoinRefusal::Subject);
        }
        let policy = bind_native(row, observation, diagnostic)?;
        if observation["layer"] != "accepted" {
            return Err(JoinRefusal::Layer);
        }
        let relations: Vec<_> = self
            .reading
            .relations()
            .iter()
            .filter(|relation| relation.event == event)
            .cloned()
            .collect();
        if relations.len()
            != 2 * self
                .reading
                .leaves()
                .iter()
                .filter(|leaf| leaf.event == event)
                .count()
        {
            return Err(JoinRefusal::Subject);
        }
        Ok(MemberExecutionAndRelationEvidence {
            subject: row.digest,
            event,
            relations,
            policy,
        })
    }
}

pub fn bind_native(
    row: &CaptureRow,
    observed: &Value,
    diagnostic: &str,
) -> Result<FinalBytePolicyObservation, JoinRefusal> {
    let _span = tracing::debug_span!("bind_final_native", schema=?row.schema).entered();
    let hex = crate::maturity_observer::encode_hex;
    if sha256(&row.submitted) != row.digest || observed["digest"] != hex(&row.digest) {
        return Err(JoinRefusal::Subject);
    }
    if observed["layer"] != serde_json::to_value(row.schema.outcome()).unwrap() {
        return Err(JoinRefusal::Layer);
    }
    let accepted = row.schema.outcome()
        == target_elements_conformance::protocol::ObservedOutcomeLayer::Accepted;
    let (route, relay, reason) = expectation(row.schema, accepted);
    if observed["route"] != route || observed["relay_allowed"] != relay {
        return Err(JoinRefusal::Route);
    }
    if observed["relay_detail"].as_str() != reason {
        return Err(JoinRefusal::Reason);
    }
    let verdict = if accepted {
        None
    } else if row.schema == CaptureSchema::Flagged {
        Some("bad-pegin-witness")
    } else {
        reason
    };
    if observed["verdict_reason"].as_str() != verdict {
        return Err(JoinRefusal::Reason);
    }
    if observed["template_member"] != relay
        || (accepted && observed["inclusion_in_produced"] != true)
        || (!accepted && !observed["inclusion_in_produced"].is_null())
    {
        return Err(JoinRefusal::Route);
    }
    block_fields(row, observed, diagnostic)?;
    let tx: elements::Transaction =
        elements::encode::deserialize(&row.submitted).map_err(|_| JoinRefusal::Subject)?;
    if observed["resources"]["transaction_weight"].as_u64() != Some(tx.weight() as u64) {
        return Err(JoinRefusal::Weight);
    }
    if accepted {
        if observed["readback"]["transaction_id"] != tx.txid().to_string()
            || observed["readback"]["witness_transaction_id"] != tx.wtxid().to_string()
            || observed["accepted_txid"] != tx.txid().to_string()
        {
            return Err(JoinRefusal::Readback);
        }
    } else if !observed["readback"].is_null() || !observed["accepted_txid"].is_null() {
        return Err(JoinRefusal::Readback);
    }
    for field in [
        "script_bytes",
        "initial_stack_items",
        "peak_stack_items",
        "peak_altstack_items",
        "maximum_element_bytes",
        "validation_budget_used",
    ] {
        if !observed["resources"][field].is_null() {
            return Err(JoinRefusal::Dimension);
        }
    }
    let policy = &observed["policy"];
    if !policy.is_object()
        || policy["unit"] != "BTC/kvB"
        || ["relay", "mempool", "block"]
            .iter()
            .any(|name| policy[*name] != "0.00000100")
        || policy["relay_provenance"] != "reported by the node"
        || policy["block_provenance"] != "declared by the pinned source, no override"
        || policy["block_identifier"] != "DEFAULT_BLOCK_MIN_TX_FEE"
    {
        return Err(JoinRefusal::Policy);
    }
    let structural = row.boundary == CaptureBoundary::StructuralEnvelope;
    if structural != (row.schema == CaptureSchema::ScalingMerge(1565))
        || (structural && row.sealed_control)
    {
        return Err(JoinRefusal::Structural);
    }
    Ok(FinalBytePolicyObservation {
        subject: row.digest,
        route: route.into(),
        relay_allowed: relay,
        structural,
    })
}

fn block_fields(row: &CaptureRow, observed: &Value, diagnostic: &str) -> Result<(), JoinRefusal> {
    let (error, text) = match row.schema {
        CaptureSchema::FundingPoint(1 | 13) => (
            "unclassified",
            "TestBlockValidity failed: bad-txns-in-ne-out, value in != value out",
        ),
        CaptureSchema::FundingPoint(2 | 14) => (
            "unknown error",
            "mandatory-script-verify-flag-failed (unknown error), Balance check failed",
        ),
        CaptureSchema::Flagged => (
            "unclassified",
            "TestBlockValidity failed: bad-pegin-witness, Not enough stack items.",
        ),
        _ => {
            return if observed["layer"] != "accepted" && !observed["block_script_error"].is_null() {
                Err(JoinRefusal::Block)
            } else {
                Ok(())
            };
        }
    };
    if observed["block_script_error"] != error || !diagnostic.contains(text) {
        return Err(JoinRefusal::Block);
    }
    if matches!(
        row.schema,
        CaptureSchema::FundingPoint(1 | 13) | CaptureSchema::Flagged
    ) {
        let tx: elements::Transaction =
            elements::encode::deserialize(&row.submitted).map_err(|_| JoinRefusal::Subject)?;
        let expected = format!("{text} in transaction {}", tx.txid());
        if !diagnostic.lines().any(|line| line == expected) {
            return Err(JoinRefusal::Block);
        }
    }
    Ok(())
}

const fn expectation(
    schema: CaptureSchema,
    accepted: bool,
) -> (&'static str, bool, Option<&'static str>) {
    use CaptureSchema as S;
    let reason = match schema {
        S::ScalingMerge(1565) => Some("tx-size"),
        S::FundingPoint(1 | 2 | 13 | 14) => Some("bad-txns-in-ne-out"),
        S::FundingPoint(3 | 9 | 15 | 21) => Some("min relay fee not met"),
        S::FundingPoint(6 | 18) => Some("dust"),
        S::FundingPoint(4 | 16) => Some(
            "mandatory-script-verify-flag-failed (Script evaluated without error but finished with a false/empty top stack element)",
        ),
        S::FundingPoint(5 | 17) => {
            Some("mandatory-script-verify-flag-failed (Non-canonical DER signature)")
        }
        S::Flagged => Some("pegin-no-witness"),
        S::WrongParity | S::UncommittedLeaf => {
            Some("mandatory-script-verify-flag-failed (Witness program hash mismatch)")
        }
        S::KeyPathAttempt => {
            Some("mandatory-script-verify-flag-failed (Invalid Schnorr signature)")
        }
        S::LaterBorn
        | S::WrongKey
        | S::WrongScript
        | S::WrongVersion
        | S::DisplacedSuccessor
        | S::ChangedMember => {
            Some("mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)")
        }
        _ => None,
    };
    (
        if !accepted {
            "refused"
        } else if reason.is_some() {
            "direct-block"
        } else {
            "relay-and-template"
        },
        reason.is_none(),
        reason,
    )
}
