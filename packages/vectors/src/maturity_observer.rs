//! Public-input observation of one confirmed announcement through a separate acquisition process.

use std::fmt::Write as _;

use realization::{AnnouncementLeadBounds, Cycle};
use serde_json::{Value, json};
use tapscript::{
    STATE_NUMS_KEY, StateInternalKeyPolicy, StateLeafRole, StateStaticLeaf, StateStaticNode,
    StateStaticSubtree, StateWitnessSchedule, TapscriptProgram,
};
use target_elements::TargetContractVersion;
use transaction::bytes::{TargetTransaction, Txid};

use crate::maturity_closure::{OracleStateCurve, closure_target};
use crate::maturity_continuity::{MaturityByteSource, submitted_transaction_identities};
use crate::maturity_history::{StateRootEdge, validate_state_root_history};
use crate::maturity_history_report::validate_observed_root_history;
use crate::maturity_recovery::{PublicAnnouncementHandoff, PublicAnnouncementLocator};
use crate::maturity_recovery_report::validate_observed_public_recovery;

const MANIFEST_FIELDS: &[&str] = &[
    "schema",
    "chain_name",
    "chain_id",
    "genesis_id",
    "transaction_id",
    "state_output_index",
    "witness_schedule",
    "minimum_lead",
    "maximum_lead",
    "static_root",
    "internal_policy",
    "contract_revision",
    "candidate_declaration",
    "abi_declaration",
];
const READING_FIELDS: &[&str] = &[
    "chain_name",
    "genesis_id",
    "block_hash",
    "block_height",
    "active_before",
    "active_after",
    "transaction_id",
    "witness_identity",
    "raw_transaction",
    "members",
    "relay_floor",
    "mempool_floor",
    "block_floor",
    "block_provenance",
    "fee_overrides",
];

fn closed(value: &Value, fields: &[&str]) -> Result<(), String> {
    let object = value.as_object().ok_or("object-required")?;
    for field in fields {
        if !object.contains_key(*field) {
            return Err(format!("missing-public-input:{field}"));
        }
    }
    if object.len() != fields.len() {
        return Err("unknown-publication-field".to_owned());
    }
    Ok(())
}

fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value[field]
        .as_str()
        .ok_or_else(|| format!("text-required:{field}"))
}

fn number(value: &Value, field: &str) -> Result<u64, String> {
    value[field]
        .as_u64()
        .ok_or_else(|| format!("integer-required:{field}"))
}

/// Decode canonical lowercase hexadecimal public data.
/// # Errors
/// Refuses odd length or a noncanonical digit.
pub fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("noncanonical-hex".to_owned());
    }
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digit = |b: u8| {
                if b.is_ascii_digit() {
                    b - b'0'
                } else {
                    b - b'a' + 10
                }
            };
            Ok(digit(pair[0]) * 16 + digit(pair[1]))
        })
        .collect()
}

/// Encode public bytes as one canonical token.
#[must_use]
pub fn encode_hex(bytes: &[u8]) -> String {
    let mut result = String::new();
    for byte in bytes {
        let _ = write!(result, "{byte:02x}");
    }
    result
}

fn digest(value: &Value, field: &str) -> Result<[u8; 32], String> {
    decode_hex(text(value, field)?)?
        .try_into()
        .map_err(|_| format!("digest-width:{field}"))
}

/// Closed public declarations; this value contains no process configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObserverManifest(Value);

impl ObserverManifest {
    /// Read the manifest member of the sole standard-input document.
    /// # Errors
    /// Refuses malformed JSON or any missing, unknown or invalid public field.
    pub fn parse(input: &str) -> Result<Self, String> {
        let envelope: Value = serde_json::from_str(input).map_err(|_| "invalid-json")?;
        closed(&envelope, &["manifest", "reading"])?;
        Self::from_value(envelope["manifest"].clone())
    }

    /// Validate a public declaration independently of any acquired bytes.
    /// # Errors
    /// Refuses a declaration outside the prescribed public subject.
    pub fn from_value(value: Value) -> Result<Self, String> {
        closed(&value, MANIFEST_FIELDS)?;
        if number(&value, "schema")? != 1
            || number(&value, "state_output_index")? != 0
            || text(&value, "chain_name")? != "elementsregtest"
            || text(&value, "witness_schedule")? != "variable-metadata"
            || text(&value, "internal_policy")? != "state-nums"
            || text(&value, "contract_revision")? != "elements-tapscript-v2"
        {
            return Err("public-declaration-disagrees".to_owned());
        }
        for field in ["chain_id", "genesis_id", "transaction_id", "static_root"] {
            digest(&value, field)?;
        }
        AnnouncementLeadBounds::new(
            Cycle::new(number(&value, "minimum_lead")?),
            Cycle::new(number(&value, "maximum_lead")?),
        )
        .map_err(|e| format!("lead-bounds:{e:?}"))?;
        let candidate = &value["candidate_declaration"];
        closed(
            candidate,
            &[
                "witness_schedule",
                "minimum_lead",
                "maximum_lead",
                "static_root",
                "internal_policy",
                "contract_revision",
            ],
        )?;
        for field in [
            "witness_schedule",
            "minimum_lead",
            "maximum_lead",
            "static_root",
            "internal_policy",
            "contract_revision",
        ] {
            if candidate[field] != value[field] {
                return Err(format!("candidate-premise:{field}"));
            }
        }
        let abi = &value["abi_declaration"];
        closed(abi, &["witness_schedule", "contract_revision"])?;
        for field in ["witness_schedule", "contract_revision"] {
            if abi[field] != value[field] {
                return Err(format!("abi-premise:{field}"));
            }
        }
        Ok(Self(value))
    }

    /// Exact public declaration retained by the capture.
    #[must_use]
    pub const fn value(&self) -> &Value {
        &self.0
    }
}

/// Node answers acquired by the restricted helper, never a producer response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObserverNodeReader(Value);

impl ObserverNodeReader {
    /// Read the acquisition member of the sole standard-input document.
    /// # Errors
    /// Refuses missing answers or an open response shape.
    pub fn parse(input: &str) -> Result<Self, String> {
        let envelope: Value = serde_json::from_str(input).map_err(|_| "invalid-json")?;
        closed(&envelope, &["manifest", "reading"])?;
        Self::from_value(envelope["reading"].clone())
    }

    /// Admit the closed acquisition shape without yet trusting its bindings.
    /// # Errors
    /// Refuses an incomplete or extended acquisition shape.
    pub fn from_value(value: Value) -> Result<Self, String> {
        closed(&value, READING_FIELDS)?;
        Ok(Self(value))
    }
}

/// The provenance of one checkpoint fact, retained separately from its value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverFactProvenance {
    NodeAnswer,
    RecomputedFromNodeBytes,
    PublicDeclarationBoundToObservation,
    ExplicitPublicParameterProjection,
}

/// A checkpoint whose public declaration and acquired operands were compared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObserverCheckpoint {
    facts: Value,
    binding: String,
}

impl ObserverCheckpoint {
    /// The canonical chain/genesis/block/height snapshot token.
    #[must_use]
    pub fn binding(&self) -> &str {
        &self.binding
    }

    /// Values and provenance in checkpoint order.
    #[must_use]
    pub const fn facts(&self) -> &Value {
        &self.facts
    }
}

/// Checked acquisition; private fields prevent a source tag from granting observation credit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedNodeObservation {
    manifest: ObserverManifest,
    reading: ObserverNodeReader,
    handoff: PublicAnnouncementHandoff,
    checkpoint: ObserverCheckpoint,
}

impl CheckedNodeObservation {
    /// Bind all public operands, complete bytes and the active containing block.
    /// # Errors
    /// Refuses the first declaration, byte, identity, membership or checkpoint disagreement.
    #[expect(
        clippy::too_many_lines,
        reason = "The ordered public acquisition checks retain their first refusal."
    )]
    pub fn bind(manifest: &ObserverManifest, reading: &ObserverNodeReader) -> Result<Self, String> {
        use ObserverFactProvenance::{
            ExplicitPublicParameterProjection as Projection, NodeAnswer,
            PublicDeclarationBoundToObservation as Declaration, RecomputedFromNodeBytes as Bytes,
        };
        let m = &manifest.0;
        let r = &reading.0;
        for field in ["chain_name", "genesis_id"] {
            if m[field] != r[field] {
                return Err(format!("checkpoint-{field}-disagrees"));
            }
        }
        let block = text(r, "block_hash")?;
        digest(r, "block_hash")?;
        let height = number(r, "block_height")?;
        if height == 0 || height > u64::from(u32::MAX) {
            return Err("checkpoint-height-invalid".to_owned());
        }
        if r["active_before"] != r["block_hash"] || r["active_after"] != r["block_hash"] {
            return Err("checkpoint-active-block-disagrees".to_owned());
        }
        if r["transaction_id"] != m["transaction_id"] {
            return Err("checkpoint-identity-disagrees".to_owned());
        }
        let identity = text(m, "transaction_id")?;
        if !r["members"]
            .as_array()
            .ok_or("block-members-required")?
            .iter()
            .any(|v| v.as_str() == Some(identity))
        {
            return Err("checkpoint-membership-absent".to_owned());
        }
        if r["fee_overrides"] != json!([])
            || text(r, "block_floor")? != "0.00000100"
            || text(r, "block_provenance")? != "pinned-source-no-override"
        {
            return Err("fee-policy-disagrees".to_owned());
        }
        for field in ["relay_floor", "mempool_floor"] {
            if text(r, field)? != "0.00000100" {
                return Err(format!("fee-floor-disagrees:{field}"));
            }
        }
        let bytes = decode_hex(text(r, "raw_transaction")?)?;
        let transaction =
            TargetTransaction::decode(&bytes).map_err(|e| format!("transaction:{e:?}"))?;
        let ids = submitted_transaction_identities(&transaction, &bytes);
        if ids.identity().to_target_display() != identity {
            return Err("copied-transaction-bytes".to_owned());
        }
        let witness = Txid::from_target_display(text(r, "witness_identity")?)
            .map_err(|e| format!("witness-identity:{e:?}"))?;
        if witness.to_target_display() != display_transaction_digest(&bytes) {
            return Err("witness-identity-disagrees".to_owned());
        }
        let target = closure_target().map_err(|e| format!("contract:{e:?}"))?;
        if target.definition().version() != TargetContractVersion::V2 {
            return Err("contract-disagrees".to_owned());
        }
        let leaf = transaction
            .witnesses()
            .first()
            .and_then(|w| w.stack().get(7))
            .ok_or("announcement-leaf-absent")?;
        let program = TapscriptProgram::decode(&target, leaf).map_err(|e| format!("leaf:{e:?}"))?;
        let subtree = StateStaticSubtree::new(
            &target,
            Some(StateStaticNode::Leaf {
                identity: 0,
                leaf: StateStaticLeaf {
                    role: StateLeafRole::Announcement,
                    program,
                    version: target.definition().leaf_version().get(),
                },
            }),
        )
        .map_err(|e| format!("subtree:{e:?}"))?;
        if subtree.root() != &digest(m, "static_root")? {
            return Err("wrong-static-subtree".to_owned());
        }
        let binding = format!(
            "{}:{}:{block}:{height}",
            text(m, "chain_id")?,
            text(r, "genesis_id")?
        );
        let handoff = PublicAnnouncementHandoff::new(
            PublicAnnouncementLocator::new(
                MaturityByteSource::NodeInterface {
                    binding: binding.clone(),
                },
                ids.identity(),
            ),
            bytes,
            0,
            StateWitnessSchedule::VariableMetadata,
            AnnouncementLeadBounds::new(
                Cycle::new(number(m, "minimum_lead")?),
                Cycle::new(number(m, "maximum_lead")?),
            )
            .map_err(|e| format!("bounds:{e:?}"))?,
            subtree,
            StateInternalKeyPolicy::new(STATE_NUMS_KEY, &OracleStateCurve)
                .map_err(|e| format!("internal-policy:{e:?}"))?,
            TargetContractVersion::V2,
        );
        let edge = StateRootEdge::from_public_handoff(handoff.clone())
            .map_err(|e| format!("recovery:{e:?}"))?;
        let fact = |name: &str, value: Value, kind: ObserverFactProvenance| json!({"name":name,"value":value,"provenance":format!("{kind:?}")});
        let facts = json!([
            fact(
                "chain-id",
                json!({"declared":m["chain_id"],"chain":r["chain_name"],"genesis":r["genesis_id"]}),
                Projection
            ),
            fact("genesis", r["genesis_id"].clone(), NodeAnswer),
            fact("block", r["block_hash"].clone(), NodeAnswer),
            fact("height", r["block_height"].clone(), NodeAnswer),
            fact("transaction", json!(identity), Bytes),
            fact(
                "predecessor",
                json!([
                    edge.predecessor().txid().to_target_display(),
                    edge.predecessor().index().to_string()
                ]),
                Bytes
            ),
            fact(
                "successor",
                json!([
                    edge.successor().txid().to_target_display(),
                    edge.successor().index().to_string()
                ]),
                Bytes
            ),
            fact("contract", m["contract_revision"].clone(), Declaration),
            fact("candidate", m["candidate_declaration"].clone(), Projection),
            fact("abi", m["abi_declaration"].clone(), Projection),
            fact("membership", json!(identity), NodeAnswer),
            fact("active-block", r["active_after"].clone(), NodeAnswer)
        ]);
        Ok(Self {
            manifest: manifest.clone(),
            reading: reading.clone(),
            handoff,
            checkpoint: ObserverCheckpoint { facts, binding },
        })
    }

    /// Recheck the same public operands before a report selects observation residuals.
    /// # Errors
    /// Refuses any changed acquisition or handoff operand.
    pub fn revalidate(&self) -> Result<(), String> {
        let fresh = Self::bind(&self.manifest, &self.reading)?;
        if fresh != *self {
            return Err("observation-binding-disagrees".to_owned());
        }
        Ok(())
    }

    /// The handoff assembled solely from declared inputs and node answers.
    #[must_use]
    pub const fn handoff(&self) -> &PublicAnnouncementHandoff {
        &self.handoff
    }

    /// The acquired snapshot and the provenance of its facts.
    #[must_use]
    pub const fn checkpoint(&self) -> &ObserverCheckpoint {
        &self.checkpoint
    }

    fn offered(&self, bytes: &[u8], block: &str, height: u64) -> Result<(), String> {
        if bytes != self.handoff.bytes() {
            return Err("copied-transaction-bytes".to_owned());
        }
        if block != text(&self.reading.0, "block_hash")? {
            return Err("checkpoint-block-disagrees".to_owned());
        }
        if height != number(&self.reading.0, "block_height")? {
            return Err("checkpoint-height-disagrees".to_owned());
        }
        Ok(())
    }
}

fn display_transaction_digest(bytes: &[u8]) -> String {
    use target_elements_conformance::constructor::tagged::sha256;
    let mut digest = sha256(&sha256(bytes));
    digest.reverse();
    encode_hex(&digest)
}

/// Completed separation experiment and the two distinct validated report roles.
pub struct MaturityObservation(Value);

fn refusal(
    result: Result<(), String>,
    expected: &str,
    key: &str,
    supplied: &str,
) -> Result<Value, String> {
    match result {
        Err(actual) if actual == expected => {
            Ok(json!({"key":key,"supplied":supplied,"outcome":actual}))
        }
        other => Err(format!("experiment-disagrees:{key}:{other:?}")),
    }
}

/// Run the ordered separation experiment, then validate the accepted acquisition.
/// # Errors
/// Refuses acquisition failure, an unexpected negative outcome or either report failure.
pub fn observe_maturity(
    manifest: &ObserverManifest,
    reader: &ObserverNodeReader,
) -> Result<MaturityObservation, String> {
    let mut rows = Vec::new();
    let mut omitted = manifest.0.clone();
    omitted
        .as_object_mut()
        .ok_or("manifest-object")?
        .remove("transaction_id");
    rows.push(refusal(
        ObserverManifest::from_value(omitted).map(|_| ()),
        "missing-public-input:transaction_id",
        "16.13 / missing-transaction",
        "transaction locator omitted",
    )?);
    let mut foreign = manifest.0.clone();
    foreign["genesis_id"] = json!("de".repeat(32));
    let foreign = ObserverManifest::from_value(foreign)?;
    rows.push(refusal(
        CheckedNodeObservation::bind(&foreign, reader).map(|_| ()),
        "checkpoint-genesis_id-disagrees",
        "checkpoint-genesis",
        "declared genesis altered",
    )?);
    // Acquisition must be valid before an offered departure can be judged against it.
    let observed = CheckedNodeObservation::bind(manifest, reader)?;
    let block = text(&reader.0, "block_hash")?;
    let height = number(&reader.0, "block_height")?;
    let mut copied = observed.handoff.bytes().to_vec();
    let last = copied.last_mut().ok_or("transaction-empty")?;
    *last ^= 1;
    rows.push(refusal(
        observed.offered(&copied, block, height),
        "copied-transaction-bytes",
        "16.13 / copied-transaction-bytes",
        "complete offered bytes altered",
    )?);
    rows.push(refusal(
        observed.offered(observed.handoff.bytes(), &"de".repeat(32), height),
        "checkpoint-block-disagrees",
        "checkpoint-block",
        "offered block altered",
    )?);
    rows.push(refusal(
        observed.offered(observed.handoff.bytes(), block, height + 1),
        "checkpoint-height-disagrees",
        "checkpoint-height",
        "offered height altered",
    )?);
    let honest = StateRootEdge::from_public_handoff(observed.handoff.clone())
        .map_err(|e| format!("edge:{e:?}"))?;
    let accepted_cursor =
        validate_state_root_history(std::slice::from_ref(&honest), honest.predecessor())
            .map_err(|e| format!("accepted-history:{e:?}"))?;
    // The first input begins after version, witness flag and the single-input count.
    let mut changed = observed.handoff.bytes().to_vec();
    *changed.get_mut(6).ok_or("transaction-prefix-absent")? ^= 1;
    let tx = TargetTransaction::decode(&changed).map_err(|e| format!("altered-input:{e:?}"))?;
    let identity = submitted_transaction_identities(&tx, &changed).identity();
    let h = observed.handoff();
    let altered = PublicAnnouncementHandoff::new(
        PublicAnnouncementLocator::new(h.locator().source().clone(), identity),
        changed,
        h.state_output_index(),
        h.schedule(),
        h.bounds(),
        h.static_subtree().clone(),
        h.internal_key(),
        h.contract(),
    );
    let first =
        StateRootEdge::from_public_handoff(altered).map_err(|e| format!("altered-edge:{e:?}"))?;
    let edges = [first, honest.clone()];
    let result = validate_state_root_history(&edges, honest.predecessor());
    if !matches!(
        result,
        Err(
            crate::maturity_history::MaturityRootHistoryRefusal::PredecessorIsNotTheCursor {
                position: 0,
                ..
            }
        )
    ) {
        return Err(format!("equal-cursor-experiment-disagrees:{result:?}"));
    }
    rows.push(json!({"key":"equal-final-cursor / altered-predecessor-edge-refused","related":"16.8 / successor-cursor-restored-after-invalid-intermediate-edge","supplied":"altered predecessor edge followed by the honest edge","outcome":"PredecessorIsNotTheCursor","equal_final_cursor":edges.last().is_some_and(|edge|edge.successor()==accepted_cursor),"edges":edges.iter().map(|e|json!({"predecessor":[e.predecessor().txid().to_target_display(),e.predecessor().index().to_string()],"successor":[e.successor().txid().to_target_display(),e.successor().index().to_string()]})).collect::<Vec<_>>() }));
    let history = validate_observed_root_history(&observed)?;
    let recovery = validate_observed_public_recovery(&observed)?;
    rows.push(json!({"key":"accepted-observation","supplied":"unaltered manifest and node answers","outcome":"both-reports-validated","programs_agree":true}));
    Ok(MaturityObservation(
        json!({"schema":1,"manifest":manifest.0,"reading":reader.0,"checkpoint":observed.checkpoint.facts,"experiment":rows,"history_report":history.render(),"recovery_report":recovery.render()}),
    ))
}

/// Render only a successfully validated experiment.
#[must_use]
pub fn render_maturity_observation(observation: &MaturityObservation) -> String {
    observation.0.to_string()
}
