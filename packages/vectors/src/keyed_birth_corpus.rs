//! Admission of the retained keyed-birth run, without executing a node.
//!
//! Schema two records request bodies and outcome projections. The eight child
//! responses also occur in its legacy rendering; all fourteen complete responses
//! occur in the run report's Debug transcript. This reader decodes that narrow
//! archived carrier, checks both projections, and replays the original planner.
//! It never supplies a missing response from an expected outcome. Debug decoding
//! is specific to this archive, not a new executor wire protocol.
//!
//! Admission establishes the four recorded subjects only. It supplies neither
//! issuance/peg-in coverage nor native stack metrics, and does not dispose of the
//! planning gate. The separate pinned-source reading remains a source reading.

use std::collections::BTreeMap;

use serde_json::{Map, Value};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    TargetContractVersion, reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationSubject,
};

use crate::keyed_birth::{
    BirthCampaignManifest, BirthFeeFloor, BirthPolicy, BirthSubjectId, PolicyFactOrigin,
    Wave1ExecutableProvenance, Wave1ExecutorBinding, Wave1TargetPacket,
};
use crate::keyed_birth_native::{
    BirthCampaignPlanner, BirthCaptureJournal, BirthNativeRefusal, BirthObservationClass,
    classify_birth_subject, replay_birth_capture,
};

const CAPTURE_NAME: &str = "ce8d7525556e.report.capture";
const TIMING_NAME: &str = "ce8d7525556e.report.timing";
const CAPTURE: &str =
    include_str!("../fixtures/keyed-birth-run-of-record/ce8d7525556e.report.capture");
const TIMING: &str =
    include_str!("../fixtures/keyed-birth-run-of-record/ce8d7525556e.report.timing");
const CAPTURE_TIMING: &str =
    include_str!("../fixtures/keyed-birth-run-of-record/ce8d7525556e.report.capture.timing");
const REPORT: &str = include_str!("../fixtures/keyed-birth-run-of-record/RUN-REPORT");
const MANIFEST: &str = include_str!("../fixtures/keyed-birth-run-of-record/MANIFEST.sha256");
const PINS: [&str; 5] = [
    "e5c39b3fe1a597cc07be8cd97704c0cec9457b2c9d948971a8973e179f15be34",
    "07b86f36ccbe84517f1d6dba8f4c6962dd750d3a0657da29bec6416d23f784ab",
    "88fc195b97991d3099513822e690cf5dda7172019bc15f6a61b7c19115efdbb2",
    "e0a063bae7c04620189872ef3602b49e542ac7cd7704dc17ba01b5ef0c8d3b73",
    "ed6be8da16fe0ca9789af46d8fab534a983b31b2f549980999dcbeeb7a45bdc3",
];

/// One of the independent admission checks failed; no subject is promoted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BirthAdmissionRefusal {
    /// Manifest, sidecar or capture-content digest disagrees.
    Digest,
    /// A required archive dimension is absent, duplicated or malformed.
    Dimension,
    /// The reviewed target, deployment, executable or policy binding disagrees.
    Binding,
    /// A report response disagrees with a capture projection or replay.
    Replay,
    /// A control is not mined or a refusal has no unique born-first site.
    Subject,
}
type Result<T> = std::result::Result<T, BirthAdmissionRefusal>;

/// Five retained files; admission does not write them or rewrite capture standing.
#[derive(Clone, Copy)]
pub struct BirthArchive<'a> {
    /// Schema-two request/outcome capture.
    pub capture: &'a str,
    /// Outer test timing.
    pub timing: &'a str,
    /// Ceremony timing sidecar, separately pinned because the manifest omits it.
    pub capture_timing: &'a str,
    /// Complete executor response transcript and eligibility report.
    pub report: &'a str,
    /// Original three-entry SHA-256 manifest.
    pub manifest: &'a str,
}

/// The exact admitted campaign, with replayed records available for audit.
pub struct AdmittedBirthCampaign {
    manifest: BirthCampaignManifest,
    planner: BirthCampaignPlanner,
    policy_facts: Vec<BirthPolicyFact>,
}

/// One admitted policy fact with its evidence origin, separate from consensus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BirthPolicyFact {
    /// Field in the rebuilt packet's policy.
    pub field: &'static str,
    /// Exact recorded value (fee floors use satoshis per kvB).
    pub value: Value,
    /// Observed response versus configured or source-declared premise.
    pub origin: PolicyFactOrigin,
    /// Retained location establishing this origin.
    pub source: &'static str,
}

impl AdmittedBirthCampaign {
    /// Rebuilt target packet and coordinate census.
    #[must_use]
    pub const fn manifest(&self) -> &BirthCampaignManifest {
        &self.manifest
    }

    /// Retained, replayed construction and observations.
    #[must_use]
    pub const fn planner(&self) -> &BirthCampaignPlanner {
        &self.planner
    }

    /// Origins for all six policy fields, including selections and startup args.
    #[must_use]
    pub fn policy_facts(&self) -> &[BirthPolicyFact] {
        &self.policy_facts
    }

    /// Classification after admission has required all four exact outcomes.
    #[must_use]
    pub fn classification(&self, subject: BirthSubjectId) -> BirthObservationClass {
        classify_birth_subject(&self.planner, subject)
    }
}

/// Admit the bundled run after checking all five independent node-side digests.
/// # Errors
/// Returns the first failed archive, binding, replay or subject check.
pub fn admit_run_of_record() -> Result<AdmittedBirthCampaign> {
    for (bytes, pin) in [CAPTURE, TIMING, CAPTURE_TIMING, REPORT, MANIFEST]
        .into_iter()
        .zip(PINS)
    {
        require(
            hex(&sha256(bytes.as_bytes())) == pin,
            BirthAdmissionRefusal::Digest,
        )?;
    }
    admit_birth_archive(BirthArchive {
        capture: CAPTURE,
        timing: TIMING,
        capture_timing: CAPTURE_TIMING,
        report: REPORT,
        manifest: MANIFEST,
    })
}

/// Check an archive against this run's bindings and replay every frozen exchange.
///
/// The public reader permits semantic negative tests with resealed envelopes;
/// callers selecting the immutable run should use [`admit_run_of_record`].
/// # Errors
/// Refuses incomplete archives, changed bindings, broken replay or open subjects.
pub fn admit_birth_archive(archive: BirthArchive<'_>) -> Result<AdmittedBirthCampaign> {
    let _span = tracing::info_span!("admit_keyed_birth_archive").entered();
    let result = admit(archive);
    tracing::info!(admitted = result.is_ok(), refusal = ?result.as_ref().err(), "birth admission complete");
    result
}

fn admit(archive: BirthArchive<'_>) -> Result<AdmittedBirthCampaign> {
    let expected_manifest = format!(
        "{}  {CAPTURE_NAME}\n{}  {TIMING_NAME}\n{}  RUN-REPORT\n",
        hex(&sha256(archive.capture.as_bytes())),
        hex(&sha256(archive.timing.as_bytes())),
        hex(&sha256(archive.report.as_bytes()))
    );
    require(
        archive.manifest == expected_manifest
            && archive.capture_timing == CAPTURE_TIMING
            && archive.timing == TIMING,
        BirthAdmissionRefusal::Digest,
    )?;
    let (content, footer) = archive
        .capture
        .split_once("capture-content-sha256 ")
        .ok_or(BirthAdmissionRefusal::Dimension)?;
    require(
        footer
            == format!(
                "{}\nnative-capture-end report\n",
                hex(&sha256(content.as_bytes()))
            ),
        BirthAdmissionRefusal::Digest,
    )?;
    let (header, operations) = content
        .split_once("operation 0 begin\n")
        .ok_or(BirthAdmissionRefusal::Dimension)?;
    let reference_header = CAPTURE
        .split_once("operation 0 begin\n")
        .ok_or(BirthAdmissionRefusal::Dimension)?
        .0;
    // This run has a fixed environment, capability census and deployment. The
    // comparison also rejects unknown, duplicated and omitted header dimensions.
    require(header == reference_header, BirthAdmissionRefusal::Binding)?;
    let (operation_text, tail) = operations
        .split_once("legacy-rendering ")
        .ok_or(BirthAdmissionRefusal::Dimension)?;
    let (legacy, ending) = tail
        .split_once('\n')
        .ok_or(BirthAdmissionRefusal::Dimension)?;
    require(
        ending == "terminal-state complete\nrun-id-input end\n",
        BirthAdmissionRefusal::Dimension,
    )?;
    let payload: Value = serde_json::from_str(&decode_text(legacy)?)
        .map_err(|_| BirthAdmissionRefusal::Dimension)?;
    require(
        payload["keyed_birth_capture_schema"] == 1
            && payload["eligible_for_review"] == true
            && payload["admitted"] == false
            && payload["construction_refusal"] == "None"
            && payload["replay_refusal"] == "None",
        BirthAdmissionRefusal::Dimension,
    )?;
    let responses = report_responses(archive.report)?;
    let journal = decode_operations(operation_text, responses)?;
    let packet = packet(header, &payload, &journal)?;
    let planner = replay_birth_capture(packet.target(), &journal)
        .map_err(|_| BirthAdmissionRefusal::Replay)?;
    let manifest = planner
        .manifest(packet)
        .map_err(|_| BirthAdmissionRefusal::Binding)?;
    let rows = payload["rows"]
        .as_array()
        .ok_or(BirthAdmissionRefusal::Dimension)?;
    require(rows.len() == 4, BirthAdmissionRefusal::Dimension)?;
    for (row, record) in rows.iter().zip(planner.records()) {
        let expected = if record.subject.id.control().is_some() {
            BirthObservationClass::BornFirstEqualVerify
        } else {
            BirthObservationClass::MinedControl
        };
        require(
            classify_birth_subject(&planner, record.subject.id) == expected,
            BirthAdmissionRefusal::Subject,
        )?;
        require(
            row["record"] == record.capture_json() && row["class"] == format!("{expected:?}"),
            BirthAdmissionRefusal::Subject,
        )?;
    }
    let policy_facts = policy_facts(manifest.packet().policy());
    Ok(AdmittedBirthCampaign {
        manifest,
        planner,
        policy_facts,
    })
}

fn policy_facts(policy: &BirthPolicy) -> Vec<BirthPolicyFact> {
    vec![
        BirthPolicyFact {
            field: "representation",
            value: Value::String(policy.representation.clone()),
            origin: PolicyFactOrigin::Declared,
            source: "legacy-rendering executable_provenance.representation",
        },
        BirthPolicyFact {
            field: "fee_policy",
            value: Value::String(policy.fee_policy.clone()),
            origin: PolicyFactOrigin::Declared,
            source: "legacy-rendering executable_provenance.fee_policy",
        },
        BirthPolicyFact {
            field: "startup_arguments",
            value: serde_json::json!(policy.startup_arguments),
            origin: PolicyFactOrigin::Observed,
            source: "all fourteen response-detail policy.start_arguments",
        },
        BirthPolicyFact {
            field: "relay",
            value: Value::from(policy.relay.satoshis_per_kvb),
            origin: policy.relay.origin,
            source: "all fourteen response-detail policy.relay and relay_provenance",
        },
        BirthPolicyFact {
            field: "mempool",
            value: Value::from(policy.mempool.satoshis_per_kvb),
            origin: policy.mempool.origin,
            source: "all fourteen response-detail policy.mempool and relay_provenance",
        },
        BirthPolicyFact {
            field: "block",
            value: Value::from(policy.block.satoshis_per_kvb),
            origin: policy.block.origin,
            source: "all fourteen response-detail policy.block, block_identifier and block_provenance",
        },
    ]
}

struct Journal(Vec<(OperationStep, NativeOperationResponse)>);
impl BirthCaptureJournal for Journal {
    fn birth_exchanges(
        &self,
    ) -> std::result::Result<Vec<(OperationStep, NativeOperationResponse)>, BirthNativeRefusal>
    {
        Ok(self.0.clone())
    }
}

fn decode_operations(
    text: &str,
    mut responses: BTreeMap<String, NativeOperationResponse>,
) -> Result<Journal> {
    let mut lines = text.lines();
    let mut exchanges = Vec::new();
    for ordinal in 0..14 {
        if ordinal != 0 {
            exact(&mut lines, &format!("operation {ordinal} begin"))?;
        }
        for (field, value) in [
            ("operation-id", format!("operation-{ordinal}")),
            ("request-id", format!("request-{ordinal}")),
        ] {
            require(
                decode_text(field_line(&mut lines, field)?)? == value,
                BirthAdmissionRefusal::Replay,
            )?;
        }
        exact(&mut lines, "request-role auxiliary")?;
        let bytes = decode_bytes(field_line(&mut lines, "request-bytes")?)?;
        let subject: OperationSubject =
            serde_json::from_str(&decode_text(field_line(&mut lines, "request-subject")?)?)
                .map_err(|_| BirthAdmissionRefusal::Dimension)?;
        let step_name = format!("birth-{}-{}", ordinal / 7, ordinal % 7);
        let response = responses
            .remove(&step_name)
            .ok_or(BirthAdmissionRefusal::Replay)?;
        let step = OperationStep::new(&step_name, subject);
        require(
            step.case() == &response.case && response.schema == 8,
            BirthAdmissionRefusal::Replay,
        )?;
        let expected_bytes = match step.subject() {
            OperationSubject::Submission(s) => s.transaction_bytes.as_slice(),
            _ => &[],
        };
        require(bytes == expected_bytes, BirthAdmissionRefusal::Replay)?;
        for (field, value) in [
            ("response-id", format!("response-{ordinal}")),
            ("response-request-id", format!("request-{ordinal}")),
            ("response-operation-id", format!("operation-{ordinal}")),
        ] {
            require(
                decode_text(field_line(&mut lines, field)?)? == value,
                BirthAdmissionRefusal::Replay,
            )?;
        }
        let (verdict, layer) = match response.observed_layer {
            ObservedOutcomeLayer::Accepted => ("accepted", "accepted"),
            ObservedOutcomeLayer::ScriptPathRejection => ("refused", "script-path-rejection"),
            _ => return Err(BirthAdmissionRefusal::Subject),
        };
        exact(&mut lines, &format!("response-verdict {verdict}"))?;
        exact(&mut lines, &format!("response-layer {layer}"))?;
        exact(
            &mut lines,
            &format!(
                "response-target-identity {}",
                response.accepted_txid.as_deref().unwrap_or("none")
            ),
        )?;
        require(
            Some(decode_text(field_line(&mut lines, "response-detail")?)?)
                == response.observed_detail,
            BirthAdmissionRefusal::Replay,
        )?;
        for field in [
            "attribution-control-request-id",
            "attribution-control-identity",
            "mutation-kind",
            "mutation-locator",
            "projection-input",
        ] {
            exact(&mut lines, &format!("{field} none"))?;
        }
        exact(&mut lines, &format!("operation {ordinal} end"))?;
        exchanges.push((step, response));
    }
    require(
        lines.next().is_none() && responses.is_empty(),
        BirthAdmissionRefusal::Dimension,
    )?;
    Ok(Journal(exchanges))
}

fn packet(header: &str, payload: &Value, journal: &Journal) -> Result<Wave1TargetPacket> {
    let text = |key| decode_text(single_field(header, key)?);
    let target = reviewed_elements_tapscript().map_err(|_| BirthAdmissionRefusal::Binding)?;
    let network = text("deployment-network-id")?;
    let genesis = text("deployment-genesis-id")?;
    require(
        network == text("environment-network-id")? && genesis == text("environment-genesis-id")?,
        BirthAdmissionRefusal::Binding,
    )?;
    let deployment = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            TargetContractVersion::V2,
            DeploymentEnvironment::Development,
            digest(&network)?,
            digest(&genesis)?,
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .map_err(|_| BirthAdmissionRefusal::Binding)?;
    let provenance = &payload["executable_provenance"];
    let reference_legacy = single_field(CAPTURE, "legacy-rendering")?;
    let reference: Value = serde_json::from_str(&decode_text(reference_legacy)?)
        .map_err(|_| BirthAdmissionRefusal::Dimension)?;
    require(
        *provenance == reference["executable_provenance"],
        BirthAdmissionRefusal::Binding,
    )?;
    let tip = text("handshake-intended-executed-tip")?;
    let executable_sha = string(provenance, "executable_sha256")?;
    let provenance_record = string(provenance, "provenance_record")?;
    require(
        string(provenance, "intended_tip")? == tip
            && provenance_record.contains(&tip)
            && provenance_record.contains(&executable_sha),
        BirthAdmissionRefusal::Binding,
    )?;
    let first = journal.0.first().ok_or(BirthAdmissionRefusal::Dimension)?;
    let detail = response_detail(&first.1)?;
    let policy = &detail["policy"];
    require(
        *policy == reference["rows"][0]["record"]["policy_and_startup"],
        BirthAdmissionRefusal::Binding,
    )?;
    for (_, response) in &journal.0 {
        require(
            response_detail(response)?["policy"] == *policy,
            BirthAdmissionRefusal::Binding,
        )?;
    }
    require(
        policy["unit"] == "BTC/kvB"
            && policy["relay_provenance"] == "reported by the node"
            && policy["block_provenance"] == "declared by the pinned source, no override"
            && policy["block_identifier"] == "DEFAULT_BLOCK_MIN_TX_FEE",
        BirthAdmissionRefusal::Binding,
    )?;
    let floor = |key, origin, source| -> Result<BirthFeeFloor> {
        // The retained run reports exactly one satoshi per ten vbytes. Reject
        // other amounts rather than round an arbitrary decimal through f64.
        require(policy[key] == "0.00000100", BirthAdmissionRefusal::Binding)?;
        Ok(BirthFeeFloor {
            satoshis_per_kvb: 100,
            origin,
            source,
        })
    };
    Wave1TargetPacket::bind(target, deployment, Wave1ExecutableProvenance {
        node_name: text("handshake-node-name")?, node_version: text("handshake-node-version")?,
        binary_revision: text("handshake-binary-reported-revision")?, intended_tip: tip,
        framework_revision: text("handshake-framework-revision")?,
        executable_path: string(provenance, "executable_path")?, executable_sha256: digest(&executable_sha)?,
        provenance_record: string(provenance, "provenance_path")?,
    }, Wave1ExecutorBinding { chain: text("environment-chain")?, protocol_schema: 8,
        adapter_name: text("handshake-adapter-name")?, adapter_version: text("handshake-adapter-version")?,
    }, BirthPolicy {
        representation: string(provenance, "representation")?, fee_policy: string(provenance, "fee_policy")?,
        startup_arguments: serde_json::from_value(policy["start_arguments"].clone()).map_err(|_| BirthAdmissionRefusal::Dimension)?,
        relay: floor("relay", PolicyFactOrigin::Observed, "retained response policy.relay; reported by the node".into())?,
        mempool: floor("mempool", PolicyFactOrigin::Observed, "retained response policy.mempool; reported by the node".into())?,
        block: floor("block", PolicyFactOrigin::Declared, "retained policy DEFAULT_BLOCK_MIN_TX_FEE; declared by the pinned source, no override".into())?,
    }).map_err(|_| BirthAdmissionRefusal::Binding)
}

fn response_detail(response: &NativeOperationResponse) -> Result<Value> {
    serde_json::from_str(
        response
            .observed_detail
            .as_deref()
            .ok_or(BirthAdmissionRefusal::Dimension)?,
    )
    .map_err(|_| BirthAdmissionRefusal::Dimension)
}

fn report_responses(report: &str) -> Result<BTreeMap<String, NativeOperationResponse>> {
    require(report.starts_with("keyed-birth-run-schema 1\nwall-seconds ")
        && report.contains("\neligible-for-review true\nadmitted false\nexecutor-result Ok(ExecutionTranscript {")
        && report.ends_with("replay-result None\n"), BirthAdmissionRefusal::Dimension)?;
    let mut rest = report;
    let mut responses = BTreeMap::new();
    while let Some((_, tail)) = rest.split_once("NativeOperationResponse {") {
        let mut parser = DebugValue(tail);
        let value = parser.object()?;
        let response: NativeOperationResponse =
            serde_json::from_value(value).map_err(|_| BirthAdmissionRefusal::Dimension)?;
        require(
            responses
                .insert(response.case.step.clone(), response)
                .is_none(),
            BirthAdmissionRefusal::Dimension,
        )?;
        rest = parser.0;
    }
    require(responses.len() == 14, BirthAdmissionRefusal::Dimension)?;
    Ok(responses)
}

// Closed decoder for the Debug response carrier actually retained by this run.
// Conversion through the protocol's deny_unknown_fields wire types is deliberate.
struct DebugValue<'a>(&'a str);
impl DebugValue<'_> {
    fn trim(&mut self) {
        self.0 = self.0.trim_start();
    }
    fn take(&mut self, token: &str) -> Result<()> {
        self.trim();
        self.0 = self
            .0
            .strip_prefix(token)
            .ok_or(BirthAdmissionRefusal::Dimension)?;
        Ok(())
    }
    fn word(&mut self) -> Result<&str> {
        self.trim();
        let end = self
            .0
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .unwrap_or(self.0.len());
        require(end != 0, BirthAdmissionRefusal::Dimension)?;
        let (word, rest) = self.0.split_at(end);
        self.0 = rest;
        Ok(word)
    }
    fn object(&mut self) -> Result<Value> {
        let mut object = Map::new();
        loop {
            self.trim();
            if self.0.starts_with('}') {
                self.take("}")?;
                return Ok(Value::Object(object));
            }
            let key = self.word()?.to_owned();
            self.take(":")?;
            let value = self.value()?;
            require(
                object.insert(key, value).is_none(),
                BirthAdmissionRefusal::Dimension,
            )?;
            self.trim();
            if !self.0.starts_with('}') {
                self.take(",")?;
            }
        }
    }
    fn value(&mut self) -> Result<Value> {
        self.trim();
        if self.0.starts_with('"') {
            let mut stream = serde_json::Deserializer::from_str(self.0).into_iter::<String>();
            let value = stream
                .next()
                .ok_or(BirthAdmissionRefusal::Dimension)?
                .map_err(|_| BirthAdmissionRefusal::Dimension)?;
            self.0 = &self.0[stream.byte_offset()..];
            return Ok(Value::String(value));
        }
        if self.0.starts_with('[') {
            self.take("[")?;
            let mut values = Vec::new();
            loop {
                self.trim();
                if self.0.starts_with(']') {
                    self.take("]")?;
                    return Ok(Value::Array(values));
                }
                values.push(self.value()?);
                self.trim();
                if !self.0.starts_with(']') {
                    self.take(",")?;
                }
            }
        }
        let word = self.word()?.to_owned();
        match word.as_str() {
            "None" => Ok(Value::Null),
            "Some" => {
                self.take("(")?;
                let value = self.value()?;
                self.take(")")?;
                Ok(value)
            }
            "OperationCaseId"
            | "NativeResourceObservation"
            | "MinedFundingReadback"
            | "FundedOutput"
            | "WireOutpoint" => {
                self.take("{")?;
                self.object()
            }
            "Accepted" => Ok(Value::String("accepted".into())),
            "ScriptPathRejection" => Ok(Value::String("script_path_rejection".into())),
            "FundSponsor" => Ok(Value::String("fund_sponsor".into())),
            "SignSponsor" => Ok(Value::String("sign_sponsor".into())),
            "Submit" => Ok(Value::String("submit".into())),
            _ => word
                .parse::<u64>()
                .map(Value::from)
                .map_err(|_| BirthAdmissionRefusal::Dimension),
        }
    }
}

const fn require(condition: bool, refusal: BirthAdmissionRefusal) -> Result<()> {
    if condition { Ok(()) } else { Err(refusal) }
}
fn string(value: &Value, key: &str) -> Result<String> {
    value[key]
        .as_str()
        .map(str::to_owned)
        .ok_or(BirthAdmissionRefusal::Dimension)
}
fn single_field<'a>(text: &'a str, key: &str) -> Result<&'a str> {
    let prefix = format!("{key} ");
    let mut matches = text.lines().filter_map(|line| line.strip_prefix(&prefix));
    let value = matches.next().ok_or(BirthAdmissionRefusal::Dimension)?;
    require(matches.next().is_none(), BirthAdmissionRefusal::Dimension)?;
    Ok(value)
}
fn field_line<'a>(lines: &mut std::str::Lines<'a>, field: &str) -> Result<&'a str> {
    lines
        .next()
        .and_then(|line| line.strip_prefix(&format!("{field} ")))
        .ok_or(BirthAdmissionRefusal::Dimension)
}
fn exact(lines: &mut std::str::Lines<'_>, expected: &str) -> Result<()> {
    require(
        lines.next() == Some(expected),
        BirthAdmissionRefusal::Dimension,
    )
}
fn decode_bytes(value: &str) -> Result<Vec<u8>> {
    let (length, bytes) = value
        .split_once(' ')
        .ok_or(BirthAdmissionRefusal::Dimension)?;
    let decoded = unhex(bytes)?;
    require(
        length.parse::<usize>().ok() == Some(decoded.len()),
        BirthAdmissionRefusal::Dimension,
    )?;
    Ok(decoded)
}
fn decode_text(value: &str) -> Result<String> {
    String::from_utf8(decode_bytes(value)?).map_err(|_| BirthAdmissionRefusal::Dimension)
}
fn digest(value: &str) -> Result<[u8; 32]> {
    unhex(value)?
        .try_into()
        .map_err(|_| BirthAdmissionRefusal::Dimension)
}
fn unhex(value: &str) -> Result<Vec<u8>> {
    require(
        value.len().is_multiple_of(2) && value.is_ascii(),
        BirthAdmissionRefusal::Dimension,
    )?;
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digit = |b: u8| match b {
                b'0'..=b'9' => Ok(b - b'0'),
                b'a'..=b'f' => Ok(b - b'a' + 10),
                _ => Err(BirthAdmissionRefusal::Dimension),
            };
            Ok(digit(pair[0])? * 16 + digit(pair[1])?)
        })
        .collect()
}
fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    bytes
        .iter()
        .flat_map(|byte| {
            [
                char::from(DIGITS[usize::from(byte >> 4)]),
                char::from(DIGITS[usize::from(byte & 15)]),
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Archive {
        capture: String,
        report: String,
        manifest: String,
    }
    impl Archive {
        fn accepted() -> Self {
            let archive = Self {
                capture: CAPTURE.into(),
                report: REPORT.into(),
                manifest: MANIFEST.into(),
            };
            assert!(archive.admit().is_ok());
            archive
        }
        fn admit(&self) -> Result<AdmittedBirthCampaign> {
            admit_birth_archive(BirthArchive {
                capture: &self.capture,
                timing: TIMING,
                capture_timing: CAPTURE_TIMING,
                report: &self.report,
                manifest: &self.manifest,
            })
        }
        fn seal(&mut self) {
            let content = self
                .capture
                .split_once("capture-content-sha256 ")
                .expect("checksum field")
                .0
                .to_owned();
            self.capture = format!(
                "{content}capture-content-sha256 {}\nnative-capture-end report\n",
                hex(&sha256(content.as_bytes()))
            );
            self.manifest = format!(
                "{}  {CAPTURE_NAME}\n{}  {TIMING_NAME}\n{}  RUN-REPORT\n",
                hex(&sha256(self.capture.as_bytes())),
                hex(&sha256(TIMING.as_bytes())),
                hex(&sha256(self.report.as_bytes()))
            );
        }
        fn payload(&mut self, edit: impl FnOnce(&mut Value)) {
            let old = single_field(&self.capture, "legacy-rendering")
                .expect("legacy field")
                .to_owned();
            let mut value: Value =
                serde_json::from_str(&decode_text(&old).expect("text")).expect("JSON");
            edit(&mut value);
            let new = value.to_string();
            self.capture = self.capture.replace(
                &format!("legacy-rendering {old}"),
                &format!("legacy-rendering {} {}", new.len(), hex(new.as_bytes())),
            );
            self.seal();
        }
        fn response(&mut self, ordinal: usize, edit: impl FnOnce(&mut NativeOperationResponse)) {
            let name = format!("birth-{}-{}", ordinal / 7, ordinal % 7);
            let mut responses = report_responses(&self.report).expect("responses");
            let response = responses.get_mut(&name).expect("case");
            let old = format!("{response:?}");
            let old_detail = response.observed_detail.clone().expect("detail");
            edit(response);
            let new = format!("{response:?}");
            assert!(self.report.contains(&old));
            self.report = self.report.replacen(&old, &new, 1);
            let new_detail = response.observed_detail.clone().expect("detail");
            let begin = format!("operation {ordinal} begin\n");
            let end = format!("operation {ordinal} end\n");
            let (before, rest) = self.capture.split_once(&begin).expect("begin");
            let (body, after) = rest.split_once(&end).expect("end");
            let body = body.replace(
                &format!(
                    "response-detail {} {}",
                    old_detail.len(),
                    hex(old_detail.as_bytes())
                ),
                &format!(
                    "response-detail {} {}",
                    new_detail.len(),
                    hex(new_detail.as_bytes())
                ),
            );
            self.capture = format!("{before}{begin}{body}{end}{after}");
            self.payload(|payload| {
                for row in payload["rows"].as_array_mut().expect("rows") {
                    for key in ["signing_response", "submission_response"] {
                        if row["record"][key]["case"]["step"] == name {
                            row["record"][key] =
                                serde_json::to_value(&*response).expect("response JSON");
                        }
                    }
                }
            });
        }
    }

    #[test]
    fn run_of_record_admits_exact_four_subjects() {
        let admitted = admit_run_of_record().unwrap_or_else(|error| panic!("admission: {error:?}"));
        assert_eq!(admitted.manifest().subjects().len(), 4);
        for (id, expected) in [
            (BirthSubjectId::A0, BirthObservationClass::MinedControl),
            (
                BirthSubjectId::A1,
                BirthObservationClass::BornFirstEqualVerify,
            ),
            (BirthSubjectId::A2, BirthObservationClass::MinedControl),
            (
                BirthSubjectId::A3,
                BirthObservationClass::BornFirstEqualVerify,
            ),
        ] {
            assert_eq!(admitted.classification(id), expected);
        }
        let heights: Vec<_> = admitted
            .planner()
            .exchanges()
            .iter()
            .filter_map(|(_, response)| response.mined_readback.as_ref().map(|r| r.block_height))
            .collect();
        assert_eq!(heights, [3, 4, 6, 7]);
        let packet = admitted.manifest().packet();
        assert_eq!(packet.executor().protocol_schema, 8);
        assert_eq!(packet.executor().adapter_version, "2.1.0");
        assert_eq!(packet.policy().relay.origin, PolicyFactOrigin::Observed);
        assert_eq!(packet.policy().mempool.origin, PolicyFactOrigin::Observed);
        assert_eq!(packet.policy().block.origin, PolicyFactOrigin::Declared);
        assert_eq!(packet.policy().relay.satoshis_per_kvb, 100);
        assert_eq!(admitted.policy_facts().len(), 6);
        assert_eq!(
            admitted.policy_facts()[0].origin,
            PolicyFactOrigin::Declared
        );
        assert_eq!(
            admitted.policy_facts()[1].origin,
            PolicyFactOrigin::Declared
        );
        assert_eq!(
            admitted.policy_facts()[2].origin,
            PolicyFactOrigin::Observed
        );
        assert_eq!(admitted.planner().exchanges().len(), 14);
    }

    #[test]
    fn changed_capture_byte_refuses_beside_accepted_archive() {
        let mut archive = Archive::accepted();
        archive.capture = archive
            .capture
            .replacen("protocol-schema 8", "protocol-schema 7", 1);
        assert!(matches!(
            archive.admit(),
            Err(BirthAdmissionRefusal::Digest)
        ));
    }

    #[test]
    fn changed_binding_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.capture = archive.capture.replacen(
            "handshake-protocol-schema 8",
            "handshake-protocol-schema 7",
            1,
        );
        archive.seal();
        assert!(matches!(
            archive.admit(),
            Err(BirthAdmissionRefusal::Binding)
        ));
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            p["executable_provenance"]["executable_sha256"] = Value::String("00".repeat(32));
        });
        assert!(matches!(
            archive.admit(),
            Err(BirthAdmissionRefusal::Binding)
        ));
    }

    #[test]
    fn changed_control_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.response(6, |response| {
            response
                .mined_readback
                .as_mut()
                .expect("mined control")
                .raw_transaction[0] ^= 1;
        });
        // Construction replay retains child outcomes; classification checks the
        // mined control and therefore also refuses its dependent negative.
        assert_eq!(archive.admit().err(), Some(BirthAdmissionRefusal::Subject));
    }

    #[test]
    fn refusal_at_another_site_remains_unadmitted() {
        let mut archive = Archive::accepted();
        archive.response(4, |response| {
            response.observed_detail = response
                .observed_detail
                .as_ref()
                .map(|s| s.replace("OP_EQUALVERIFY", "OP_CHECKSIG"));
        });
        assert!(matches!(
            archive.admit(),
            Err(BirthAdmissionRefusal::Subject)
        ));
    }

    #[test]
    fn missing_dimension_refuses_after_resealing() {
        let mut archive = Archive::accepted();
        archive.payload(|p| {
            p["rows"][0]["record"]
                .as_object_mut()
                .expect("record")
                .remove("consensus_evidence");
        });
        assert!(matches!(
            archive.admit(),
            Err(BirthAdmissionRefusal::Subject)
        ));
    }

    #[test]
    fn digest_mismatch_refuses_beside_accepted_archive() {
        let mut archive = Archive::accepted();
        archive.manifest.replace_range(..1, "0");
        assert!(matches!(
            archive.admit(),
            Err(BirthAdmissionRefusal::Digest)
        ));
    }
}
