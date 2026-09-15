//! Strict admission and planner replay of the native operator run of record.
//!
//! File addresses bind the archive before its closed grammars are interpreted.
//! The planner settles the recorded exchanges before any row standing is read.
//! Funding subjects are reconstructed from the retained issuance and output facts;
//! the capture retains signing contexts and submitted bytes directly.

use std::fmt::Write as _;
use std::sync::OnceLock;

use linker::CandidateDeploymentIdentity;
use target_elements_conformance::constructor::tagged;
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationSubject, TargetFundingSubject,
    TargetScriptPathSigningSubject, TargetSubmissionSubject, WireSighashProfile, WireSpentOutput,
    WireTapleaf,
};
use target_elements_conformance::test_material::PublicTestSignerHandle;
use transaction::{TargetTransaction, Txid};

use crate::maturity_operator::{EVIDENCE_ROWS, NATIVE_STEPS, OperatorEvidence};

/// SHA-256 of the exact two-entry evidence manifest.
pub const NATIVE_OPERATOR_MANIFEST_SHA256: &str =
    "d55f795837247c73b4c4e3d65d87d9f3a49b141e1236b4b8997f7fa5caff4244";
/// SHA-256 of the report binding the complete operator run.
pub const NATIVE_OPERATOR_RUN_ADDRESS: &str =
    "901c86cc0a7470d675563535dd5d78ce4b5594ace49a056a185bc9dbe1bce068";

const CEREMONY_SHA256: &str = "d88f6481a6eff197f30736f66e3a1bae2d97f51005e0cc4460d89b64e8d24dbe";

#[derive(Clone, Copy)]
struct ArchiveFile<'a> {
    name: &'a str,
    bytes: &'a [u8],
}

const FILES: [ArchiveFile<'static>; 4] = [
    ArchiveFile {
        name: "353e698f.report.capture",
        bytes: include_bytes!("../fixtures/native-operator-run-of-record/353e698f.report.capture"),
    },
    ArchiveFile {
        name: "353e698f.report.capture.timing",
        bytes: include_bytes!(
            "../fixtures/native-operator-run-of-record/353e698f.report.capture.timing"
        ),
    },
    ArchiveFile {
        name: "MANIFEST.sha256",
        bytes: include_bytes!("../fixtures/native-operator-run-of-record/MANIFEST.sha256"),
    },
    ArchiveFile {
        name: "RUN-REPORT",
        bytes: include_bytes!("../fixtures/native-operator-run-of-record/RUN-REPORT"),
    },
];
const FILE_SIZES: [usize; 4] = [76_311, 49, 187, 1_933];

/// The layer at which operator corpus admission stopped.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NativeOperatorImportRefusal {
    /// The four-file ordered census differs from the embedded archive.
    CorpusCensus,
    /// A fixed file has a different byte length.
    FileSize {
        name: String,
        expected: usize,
        actual: usize,
    },
    /// The manifest differs from its pinned SHA-256 address.
    ManifestHash,
    /// The manifest violates its closed two-line sorted grammar.
    ManifestGrammar,
    /// An embedded file differs from its manifest entry.
    ManifestDigest { name: String },
    /// The report violates its ordered grammar or fixed eligibility facts.
    RunReportGrammar,
    /// The report differs from its pinned run address.
    RunReportAddress,
    /// The capture prefix differs from its own recorded content hash.
    CaptureContentHash,
    /// A report, capture, deployment or response binding disagrees.
    CrossFileBinding { field: &'static str },
    /// The capture, payload or timing sidecar violates its closed grammar.
    TranscriptGrammar { name: String, line: usize },
    /// An operation, request or response identity does not link exactly.
    IdentifierLink { step: &'static str },
    /// The response JSON or protocol shape is invalid.
    ResponseGrammar { step: &'static str },
    /// A planned subject differs from the recorded or reconstructed subject.
    ReplaySubject { step: &'static str },
    /// Planner settlement refused a response, signature or readback.
    ReplaySettlement { step: &'static str },
    /// The accepted identity differs from the transaction's recomputed txid.
    TransactionIdentity,
    /// A recorded row differs from the standing derived by planner replay.
    RowStanding { row: &'static str },
    /// The payload's message differs from independent recomputation.
    RecomputedMessage,
}

type ImportResult<T> = Result<T, NativeOperatorImportRefusal>;
use NativeOperatorImportRefusal as Refusal;

/// Report facts admitted together with the complete corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeOperatorReportFacts {
    suite_commit: String,
    suite_tree: String,
    expected_tip: String,
    binary_revision: String,
    intended_tip: String,
    adapter_name: String,
    adapter_version: String,
    node_name: String,
    node_version: String,
    environment: String,
    network_id: String,
    genesis_id: String,
    target_contract: String,
    ceremony_digest: String,
    manifest_digest: String,
    run_address: String,
}

impl NativeOperatorReportFacts {
    /// Suite source commit read from the report.
    #[must_use]
    pub fn suite_commit(&self) -> &str {
        &self.suite_commit
    }

    /// Suite source tree read from the report.
    #[must_use]
    pub fn suite_tree(&self) -> &str {
        &self.suite_tree
    }

    /// Expected Elements source tip read from the report.
    #[must_use]
    pub fn expected_tip(&self) -> &str {
        &self.expected_tip
    }

    /// Revision reported by the executed binary.
    #[must_use]
    pub fn binary_revision(&self) -> &str {
        &self.binary_revision
    }

    /// Intended executed source tip.
    #[must_use]
    pub fn intended_tip(&self) -> &str {
        &self.intended_tip
    }

    /// Executed adapter name.
    #[must_use]
    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    /// Executed adapter version.
    #[must_use]
    pub fn adapter_version(&self) -> &str {
        &self.adapter_version
    }

    /// Executed node name.
    #[must_use]
    pub fn node_name(&self) -> &str {
        &self.node_name
    }

    /// Complete binary version banner.
    #[must_use]
    pub fn node_version(&self) -> &str {
        &self.node_version
    }

    /// Deployment environment.
    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }

    /// Printed-order deployment network identity.
    #[must_use]
    pub fn network_id(&self) -> &str {
        &self.network_id
    }

    /// Printed-order deployment genesis identity.
    #[must_use]
    pub fn genesis_id(&self) -> &str {
        &self.genesis_id
    }

    /// Deployment target contract.
    #[must_use]
    pub fn target_contract(&self) -> &str {
        &self.target_contract
    }

    /// SHA-256 of the complete capture file.
    #[must_use]
    pub fn ceremony_digest(&self) -> &str {
        &self.ceremony_digest
    }

    /// SHA-256 of the manifest bytes.
    #[must_use]
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    /// SHA-256 of the complete report bytes.
    #[must_use]
    pub fn run_address(&self) -> &str {
        &self.run_address
    }

    /// The admitted native protocol revision.
    #[must_use]
    pub const fn protocol_revision(&self) -> u32 {
        8
    }
}

/// Complete immutable operator evidence derived from the admitted bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedNativeOperatorCorpus {
    evidence: OperatorEvidence,
    report: NativeOperatorReportFacts,
    capture_suite: [String; 2],
    exchanges: Vec<(OperationSubject, NativeOperationResponse)>,
    capabilities: Vec<String>,
    recorded_message: Vec<u8>,
}

impl ValidatedNativeOperatorCorpus {
    /// The evidence produced by the planner's reviewed non-mock settlement.
    #[must_use]
    pub const fn evidence(&self) -> &OperatorEvidence {
        &self.evidence
    }

    /// The closed report's cross-checked facts.
    #[must_use]
    pub const fn report(&self) -> &NativeOperatorReportFacts {
        &self.report
    }

    /// Source commit read independently from the capture.
    #[must_use]
    pub fn capture_suite_commit(&self) -> &str {
        &self.capture_suite[0]
    }

    /// Source tree read independently from the capture.
    #[must_use]
    pub fn capture_suite_tree(&self) -> &str {
        &self.capture_suite[1]
    }

    /// Recorded responses and their replay-checked subjects in native step order.
    #[must_use]
    pub fn exchanges(&self) -> &[(OperationSubject, NativeOperationResponse)] {
        &self.exchanges
    }

    /// The exact ordered environment capability roster.
    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    /// The payload message compared with independent planner recomputation.
    #[must_use]
    pub fn recorded_message(&self) -> &[u8] {
        &self.recorded_message
    }

    /// Stable content address of the report-bound corpus.
    #[must_use]
    pub fn content_address(&self) -> &str {
        self.report.run_address()
    }
}

struct Cursor<'a> {
    name: &'static str,
    lines: Vec<&'a str>,
    index: usize,
}

impl<'a> Cursor<'a> {
    fn new(name: &'static str, bytes: &'a [u8]) -> ImportResult<Self> {
        let refusal = || Refusal::TranscriptGrammar {
            name: name.to_owned(),
            line: 1,
        };
        if bytes.contains(&b'\r') || !bytes.ends_with(b"\n") {
            return Err(refusal());
        }
        let text = std::str::from_utf8(bytes).map_err(|_| refusal())?;
        let body = text.strip_suffix('\n').ok_or_else(refusal)?;
        if body.split('\n').any(str::is_empty) {
            return Err(refusal());
        }
        Ok(Self {
            name,
            lines: body.split('\n').collect(),
            index: 0,
        })
    }

    fn refusal(&self) -> Refusal {
        Refusal::TranscriptGrammar {
            name: self.name.to_owned(),
            line: self.index + 1,
        }
    }

    fn exact(&mut self, expected: &str) -> ImportResult<()> {
        if self.lines.get(self.index).copied() != Some(expected) {
            return Err(self.refusal());
        }
        self.index += 1;
        Ok(())
    }

    fn value(&mut self, field: &str) -> ImportResult<&'a str> {
        let line = self
            .lines
            .get(self.index)
            .copied()
            .ok_or_else(|| self.refusal())?;
        let value = line
            .strip_prefix(field)
            .and_then(|tail| tail.strip_prefix(' '))
            .ok_or_else(|| self.refusal())?;
        self.index += 1;
        Ok(value)
    }

    fn bytes(&mut self, field: &str) -> ImportResult<Vec<u8>> {
        let value = self.value(field)?;
        let (length, hex) = value.split_once(' ').ok_or_else(|| self.refusal())?;
        let bytes = decode_hex(hex).ok_or_else(|| self.refusal())?;
        if number(length) != Some(bytes.len()) {
            return Err(self.refusal());
        }
        Ok(bytes)
    }

    fn text(&mut self, field: &str) -> ImportResult<String> {
        String::from_utf8(self.bytes(field)?).map_err(|_| self.refusal())
    }

    fn done(&self) -> ImportResult<()> {
        if self.index != self.lines.len() {
            return Err(self.refusal());
        }
        Ok(())
    }
}

fn number(text: &str) -> Option<usize> {
    if text.is_empty()
        || text.len() > 1 && text.starts_with('0')
        || !text.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    text.parse().ok()
}

fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2)
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let (pairs, _) = text.as_bytes().as_chunks::<2>();
    pairs
        .iter()
        .map(|pair| {
            let digit = |byte: u8| {
                if byte.is_ascii_digit() {
                    byte - b'0'
                } else {
                    byte - b'a' + 10
                }
            };
            Some((digit(pair[0]) << 4) | digit(pair[1]))
        })
        .collect()
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

fn digest(text: &str) -> Option<[u8; 32]> {
    decode_hex(text)?.try_into().ok()
}

fn source_identity(text: &str) -> bool {
    text.len() == 40 && decode_hex(text).is_some()
}

const fn binding(field: &'static str, agrees: bool) -> ImportResult<()> {
    if agrees {
        Ok(())
    } else {
        Err(Refusal::CrossFileBinding { field })
    }
}

fn validate_census(files: &[ArchiveFile<'_>], sizes: &[usize; 4]) -> ImportResult<()> {
    if files.len() != FILES.len() || files.iter().zip(FILES).any(|(a, b)| a.name != b.name) {
        return Err(Refusal::CorpusCensus);
    }
    for (file, expected) in files.iter().zip(sizes) {
        if file.bytes.len() != *expected {
            return Err(Refusal::FileSize {
                name: file.name.to_owned(),
                expected: *expected,
                actual: file.bytes.len(),
            });
        }
    }
    Ok(())
}

fn validate_manifest(files: &[ArchiveFile<'_>], expected_hash: &str) -> ImportResult<()> {
    if hex_bytes(&tagged::sha256(files[2].bytes)) != expected_hash {
        return Err(Refusal::ManifestHash);
    }
    let mut cursor =
        Cursor::new("MANIFEST.sha256", files[2].bytes).map_err(|_| Refusal::ManifestGrammar)?;
    for file in &files[..2] {
        let line = cursor
            .lines
            .get(cursor.index)
            .ok_or(Refusal::ManifestGrammar)?;
        let (hash, name) = line.split_once("  ").ok_or(Refusal::ManifestGrammar)?;
        if name != file.name || digest(hash).is_none() {
            return Err(Refusal::ManifestGrammar);
        }
        if hex_bytes(&tagged::sha256(file.bytes)) != hash {
            return Err(Refusal::ManifestDigest {
                name: name.to_owned(),
            });
        }
        cursor.index += 1;
    }
    cursor.done().map_err(|_| Refusal::ManifestGrammar)
}

fn parse_report_suite(cursor: &mut Cursor<'_>) -> ImportResult<[String; 5]> {
    cursor.exact("run-report-schema native-operator-run-report 1")?;
    cursor.exact("capture-format-schema native-operator-capture 1")?;
    let suite_commit = cursor.value("suite-commit")?.to_owned();
    let suite_tree = cursor.value("suite-tree")?.to_owned();
    if !source_identity(&suite_commit) || !source_identity(&suite_tree) {
        return Err(cursor.refusal());
    }
    cursor.exact("suite-clean yes")?;
    cursor.exact("rust-test-target maturity_operator")?;
    if cursor.text("cargo-argv")?
        != concat!(
            "/workspace/toolchains/cargo/bin/cargo test -p tripod-vectors ",
            "--test maturity_operator -- --ignored --test-threads=1"
        )
    {
        return Err(cursor.refusal());
    }
    for line in [
        "expected-test-count 1",
        "observed-test-count 1",
        "expected-ceremony-count 1",
        "observed-ceremony-count 1",
        "expected-setup-count 0",
        "observed-setup-count 0",
        "diagnostics-present yes",
        "test-passed-count 1",
        "test-failed-count 0",
        "test-ignored-count 0",
        "cargo-exit-code 0",
    ] {
        cursor.exact(line)?;
    }
    let expected_tip = cursor.value("elementsd-expected-tip")?.to_owned();
    let binary_revision = cursor.text("elementsd-binary-reported-revision")?;
    let intended_tip = cursor.text("elementsd-intended-executed-tip")?;
    if !source_identity(&expected_tip)
        || binary_revision.len() != 12
        || !expected_tip.starts_with(&binary_revision)
        || intended_tip != expected_tip
    {
        return Err(cursor.refusal());
    }
    Ok([
        suite_commit,
        suite_tree,
        expected_tip,
        binary_revision,
        intended_tip,
    ])
}

fn parse_report_body(bytes: &[u8]) -> ImportResult<NativeOperatorReportFacts> {
    let mut cursor = Cursor::new("RUN-REPORT", bytes)?;
    let [
        suite_commit,
        suite_tree,
        expected_tip,
        binary_revision,
        intended_tip,
    ] = parse_report_suite(&mut cursor)?;
    let adapter_name = cursor.text("executor-adapter-name")?;
    let adapter_version = cursor.text("executor-adapter-version")?;
    let node_name = cursor.text("node-name")?;
    let node_version = cursor.text("node-version")?;
    if adapter_name != "elements-native-executor"
        || adapter_version != "2.1.0"
        || node_name != "Elements"
        || node_version != format!("Elements Core daemon version v28.99.0-{binary_revision}")
    {
        return Err(cursor.refusal());
    }
    cursor.exact("protocol-revision 8")?;
    cursor.exact("fixture-digest-algorithm forward-v2")?;
    let environment = cursor.value("deployment-environment")?.to_owned();
    let network_id = cursor.text("deployment-network-id")?;
    let genesis_id = cursor.text("deployment-genesis-id")?;
    let target_contract = cursor.text("deployment-target-contract")?;
    if environment != "development"
        || digest(&network_id).is_none()
        || digest(&genesis_id).is_none()
        || target_contract != "elements-tapscript-v2"
    {
        return Err(cursor.refusal());
    }
    cursor.exact("ceremony-roster begin")?;
    let ceremony_digest = cursor.value("ceremony report")?.to_owned();
    cursor.exact("ceremony-roster end")?;
    let manifest_digest = cursor.value("manifest-sha256")?.to_owned();
    if digest(&ceremony_digest).is_none() || digest(&manifest_digest).is_none() {
        return Err(cursor.refusal());
    }
    cursor.exact("eligible yes")?;
    cursor.done()?;
    Ok(NativeOperatorReportFacts {
        suite_commit,
        suite_tree,
        expected_tip,
        binary_revision,
        intended_tip,
        adapter_name,
        adapter_version,
        node_name,
        node_version,
        environment,
        network_id,
        genesis_id,
        target_contract,
        ceremony_digest,
        manifest_digest,
        run_address: hex_bytes(&tagged::sha256(bytes)),
    })
}

fn parse_report(bytes: &[u8], expected_hash: &str) -> ImportResult<NativeOperatorReportFacts> {
    let report = parse_report_body(bytes).map_err(|_| Refusal::RunReportGrammar)?;
    if report.run_address != expected_hash {
        return Err(Refusal::RunReportAddress);
    }
    Ok(report)
}

const CAPABILITIES: [&str; 14] = [
    "failure-class-reporting",
    "transaction-context",
    "resource-observation",
    "tree-materialization",
    "confidential-conservation",
    "owner-authorized-normalization",
    "compound-prototype-fixtures",
    "fresh-process-lifecycle",
    "test-funding-ceremony",
    "target-transaction-submission",
    "test-sponsor-authorization",
    "test-script-path-authorization",
    "confidential-value-test-funding",
    "confidential-value-sponsor-authorization",
];

fn parse_environment(
    cursor: &mut Cursor<'_>,
    report: &NativeOperatorReportFacts,
) -> ImportResult<Vec<String>> {
    cursor.exact("environment-schema 8")?;
    cursor.exact("environment-chain 15 656c656d656e747372656774657374")?;
    binding(
        "environment-network-id",
        cursor.text("environment-network-id")? == report.network_id,
    )?;
    binding(
        "environment-genesis-id",
        cursor.text("environment-genesis-id")? == report.genesis_id,
    )?;
    for line in [
        "environment-domain-count 1",
        "environment-domain 0 tapscript supported true active true",
        "environment-leaf-count 1",
        "environment-leaf 0 196 supported true active true",
        "environment-capability-count 14",
    ] {
        cursor.exact(line)?;
    }
    let mut capabilities = Vec::new();
    for (index, expected) in CAPABILITIES.iter().enumerate() {
        let value = cursor.value("environment-capability")?;
        if value != format!("{index} {expected}") {
            return Err(cursor.refusal());
        }
        capabilities.push((*expected).to_owned());
    }
    for line in [
        "environment-funding-count 4",
        "environment-funding 0 representation explicit-asset-confidential-value",
        "environment-funding 1 custody central-public-fixtures",
        "environment-funding 2 materializer guide-ctf-deterministic-v1",
        "environment-funding 3 reproducibility byte_identity",
    ] {
        cursor.exact(line)?;
    }
    Ok(capabilities)
}

fn parse_header(
    cursor: &mut Cursor<'_>,
    report: &NativeOperatorReportFacts,
) -> ImportResult<[String; 2]> {
    cursor.exact("native-capture-schema 1")?;
    cursor.exact("ceremony-id report")?;
    if cursor.text("rust-test-name")? != "the_live_transfer_candidate_runs_against_a_real_target" {
        return Err(cursor.refusal());
    }
    let suite_commit = cursor.value("suite-commit")?.to_owned();
    let suite_tree = cursor.value("suite-tree")?.to_owned();
    binding(
        "suite-commit",
        source_identity(&suite_commit) && suite_commit == report.suite_commit,
    )?;
    binding(
        "suite-tree",
        source_identity(&suite_tree) && suite_tree == report.suite_tree,
    )?;
    cursor.exact("fixture-digest-algorithm forward-v2")?;
    cursor.exact("run-id-input begin")?;
    binding(
        "deployment-environment",
        cursor.value("deployment-environment")? == report.environment,
    )?;
    for (field, expected) in [
        ("deployment-network-id", &report.network_id),
        ("deployment-genesis-id", &report.genesis_id),
        ("deployment-target-contract", &report.target_contract),
    ] {
        binding(field, cursor.text(field)? == *expected)?;
    }
    binding(
        "protocol-revision",
        cursor.value("handshake-protocol-schema")? == "8",
    )?;
    for (field, expected) in [
        ("handshake-adapter-name", &report.adapter_name),
        ("handshake-adapter-version", &report.adapter_version),
        ("handshake-framework-revision", &report.intended_tip),
        ("handshake-node-name", &report.node_name),
        ("handshake-node-version", &report.node_version),
        (
            "handshake-binary-reported-revision",
            &report.binary_revision,
        ),
        ("handshake-intended-executed-tip", &report.intended_tip),
    ] {
        binding(field, cursor.text(field)? == *expected)?;
    }
    cursor.exact("handshake-upstream-base 0 ")?;
    cursor.exact("handshake-topic-count 0")?;
    Ok([suite_commit, suite_tree])
}

struct RecordedOperation {
    request_bytes: Vec<u8>,
    layer: ObservedOutcomeLayer,
    identity: Option<String>,
    detail: String,
}

fn parse_operation(
    cursor: &mut Cursor<'_>,
    index: usize,
    step: &'static str,
) -> ImportResult<RecordedOperation> {
    cursor.exact(&format!("operation {index} begin"))?;
    for (field, expected) in [
        ("operation-id", format!("operation-{index}")),
        ("request-id", format!("request-{index}")),
    ] {
        if cursor.text(field)? != expected {
            return Err(Refusal::IdentifierLink { step });
        }
    }
    cursor.exact("request-role auxiliary")?;
    let request_bytes = cursor.bytes("request-bytes")?;
    if (index < 4) != request_bytes.is_empty() {
        return Err(cursor.refusal());
    }
    for (field, expected) in [
        ("response-id", format!("response-{index}")),
        ("response-request-id", format!("request-{index}")),
        ("response-operation-id", format!("operation-{index}")),
    ] {
        if cursor.text(field)? != expected {
            return Err(Refusal::IdentifierLink { step });
        }
    }
    let accepted = index < 4 || index == 10;
    cursor.exact(if accepted {
        "response-verdict accepted"
    } else {
        "response-verdict refused"
    })?;
    cursor.exact(if accepted {
        "response-layer accepted"
    } else {
        "response-layer script-path-rejection"
    })?;
    let identity = cursor.value("response-target-identity")?;
    let identity = if index == 10 {
        if digest(identity).is_none() {
            return Err(cursor.refusal());
        }
        Some(identity.to_owned())
    } else {
        if identity != "none" {
            return Err(cursor.refusal());
        }
        None
    };
    let detail = cursor.text("response-detail")?;
    if accepted != detail.is_empty() {
        return Err(cursor.refusal());
    }
    for line in [
        "attribution-control-request-id none",
        "attribution-control-identity none",
        "mutation-kind none",
        "mutation-locator none",
        "projection-input none",
    ] {
        cursor.exact(line)?;
    }
    cursor.exact(&format!("operation {index} end"))?;
    Ok(RecordedOperation {
        request_bytes,
        identity,
        detail,
        layer: if accepted {
            ObservedOutcomeLayer::Accepted
        } else {
            ObservedOutcomeLayer::ScriptPathRejection
        },
    })
}

fn parse_tail(cursor: &mut Cursor<'_>, bytes: &[u8]) -> ImportResult<Vec<u8>> {
    let payload = cursor.bytes("legacy-rendering")?;
    cursor.exact("terminal-state complete")?;
    cursor.exact("run-id-input end")?;
    let prefix_len = cursor.lines[..cursor.index]
        .iter()
        .map(|line| line.len() + 1)
        .sum::<usize>();
    let hash = cursor.value("capture-content-sha256")?;
    if digest(hash).is_none() {
        return Err(cursor.refusal());
    }
    if hex_bytes(&tagged::sha256(&bytes[..prefix_len])) != hash {
        return Err(Refusal::CaptureContentHash);
    }
    cursor.exact("native-capture-end report")?;
    cursor.done()?;
    Ok(payload)
}

fn parse_signing(cursor: &mut Cursor<'_>, index: usize) -> ImportResult<OperationSubject> {
    let signer = if index == 2 {
        PublicTestSignerHandle::First
    } else {
        PublicTestSignerHandle::Third
    };
    cursor.exact(&format!("signer-handle {signer:?}"))?;
    cursor.exact("input-index 0")?;
    cursor.exact("profile AllInputsAllOutputs")?;
    let finalized_transaction = cursor.bytes("finalized-transaction")?;
    cursor.exact("spent-output-count 1")?;
    let spent_outputs = vec![WireSpentOutput {
        asset_field: cursor.bytes("spent-asset")?,
        value_field: cursor.bytes("spent-value")?,
        program: cursor.bytes("spent-program")?,
    }];
    cursor.exact("leaf-version 196")?;
    let executing_leaf = WireTapleaf {
        leaf_version: 196,
        script: cursor.bytes("leaf-script")?,
        control_block: cursor.bytes("control-block")?,
    };
    Ok(OperationSubject::ScriptPathSigning(Box::new(
        TargetScriptPathSigningSubject {
            finalized_transaction,
            input_index: 0,
            spent_outputs,
            executing_leaf,
            sighash_profile: WireSighashProfile::AllInputsAllOutputs,
            signer,
        },
    )))
}

fn funding_subject(
    response: &NativeOperationResponse,
    index: usize,
) -> ImportResult<OperationSubject> {
    let step = NATIVE_STEPS[index];
    let [coin] = response.funded_outputs.as_slice() else {
        return Err(Refusal::ReplaySubject { step });
    };
    let issue_asset = index == 0;
    if issue_asset != response.issued_asset.is_some()
        || issue_asset && response.issued_asset.as_ref() != Some(&coin.asset)
    {
        return Err(Refusal::ReplaySubject { step });
    }
    // Funding request bytes were not retained. Every reconstructed subject field
    // is checked against the planner, including the issuance flag and coin census.
    Ok(OperationSubject::Funding(Box::new(TargetFundingSubject {
        issue_asset,
        asset: if issue_asset {
            None
        } else {
            Some(coin.asset.clone())
        },
        output_program: decode_hex(&coin.script).ok_or(Refusal::ReplaySubject { step })?,
        outputs: 1,
        amount_per_output: coin.amount_satoshis,
    })))
}

struct Payload {
    exchanges: Vec<(OperationSubject, NativeOperationResponse)>,
    rows: Vec<String>,
    message: Vec<u8>,
}

fn parse_payload(bytes: &[u8], operations: &[RecordedOperation]) -> ImportResult<Payload> {
    let mut cursor = Cursor::new("operator payload", bytes)?;
    cursor.exact("operator-evidence-schema 1")?;
    cursor.exact("state-construction incomplete")?;
    cursor.exact("operator-membership external-evidence-required")?;
    let message = cursor.bytes("recomputed-message")?;
    if message.len() != 32 {
        return Err(cursor.refusal());
    }
    let mut exchanges = Vec::new();
    for ((index, name), operation) in NATIVE_STEPS.iter().enumerate().zip(operations) {
        cursor.exact(&format!("operator-step {name}"))?;
        let signing = if matches!(index, 2 | 3) {
            Some(parse_signing(&mut cursor, index)?)
        } else {
            None
        };
        let response =
            NativeOperationResponse::from_recorded_json(&cursor.bytes("validated-response")?)
                .map_err(|_| Refusal::ResponseGrammar { step: name })?;
        response
            .validate_shape()
            .map_err(|_| Refusal::ResponseGrammar { step: name })?;
        if response.schema != 8 {
            return Err(Refusal::ResponseGrammar { step: name });
        }
        if response.case.step != *name {
            return Err(Refusal::IdentifierLink { step: name });
        }
        binding(
            "response-summary",
            response.observed_layer == operation.layer
                && response.accepted_txid == operation.identity
                && response.observed_detail.as_deref().unwrap_or_default() == operation.detail,
        )?;
        let subject = match index {
            0 | 1 => funding_subject(&response, index)?,
            2 | 3 => signing.ok_or_else(|| cursor.refusal())?,
            _ => OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: operation.request_bytes.clone(),
            })),
        };
        if response.case.operation != subject.kind() {
            return Err(Refusal::IdentifierLink { step: name });
        }
        exchanges.push((subject, response));
    }
    let mut rows = Vec::new();
    for name in EVIDENCE_ROWS {
        let row = cursor.value("row")?;
        if !row.starts_with(&format!("{name} expected ")) {
            return Err(cursor.refusal());
        }
        rows.push(row.to_owned());
    }
    cursor.done()?;
    Ok(Payload {
        exchanges,
        rows,
        message,
    })
}

fn verify_transaction(
    exchanges: &[(OperationSubject, NativeOperationResponse)],
) -> ImportResult<()> {
    let (_, response) = exchanges.last().ok_or(Refusal::TransactionIdentity)?;
    let readback = response
        .mined_readback
        .as_ref()
        .ok_or(Refusal::TransactionIdentity)?;
    let transaction = TargetTransaction::decode(&readback.raw_transaction)
        .map_err(|_| Refusal::TransactionIdentity)?;
    if transaction.encode() != readback.raw_transaction {
        return Err(Refusal::TransactionIdentity);
    }
    let first = tagged::sha256(&transaction.encode_without_witness());
    let identity = Txid::from_internal(tagged::sha256(&first)).to_target_display();
    if identity != readback.transaction_id || response.accepted_txid.as_ref() != Some(&identity) {
        return Err(Refusal::TransactionIdentity);
    }
    Ok(())
}

fn compare_rows(evidence: &OperatorEvidence, payload: &Payload) -> ImportResult<()> {
    if evidence.rows.len() != payload.rows.len() {
        return Err(Refusal::CorpusCensus);
    }
    for (derived, recorded) in evidence.rows.iter().zip(&payload.rows) {
        let expected = format!(
            "{} expected {:?} observed {:?} standing {:?}",
            derived.subject, derived.expected_layer, derived.observed_layer, derived.standing
        );
        if *recorded != expected {
            return Err(Refusal::RowStanding {
                row: derived.subject,
            });
        }
    }
    if evidence.message.as_deref() != Some(payload.message.as_slice()) {
        return Err(Refusal::RecomputedMessage);
    }
    Ok(())
}

fn admit_capture(
    bytes: &[u8],
    report: NativeOperatorReportFacts,
) -> ImportResult<ValidatedNativeOperatorCorpus> {
    binding(
        "ceremony-digest",
        hex_bytes(&tagged::sha256(bytes)) == report.ceremony_digest,
    )?;
    let mut cursor = Cursor::new("operator capture", bytes)?;
    let capture_suite = parse_header(&mut cursor, &report)?;
    let capabilities = parse_environment(&mut cursor, &report)?;
    cursor.exact("digest-count 0")?;
    cursor.exact("operation-count 11")?;
    let operations = NATIVE_STEPS
        .iter()
        .enumerate()
        .map(|(index, step)| parse_operation(&mut cursor, index, step))
        .collect::<ImportResult<Vec<_>>>()?;
    let payload = parse_payload(&parse_tail(&mut cursor, bytes)?, &operations)?;
    let identity = CandidateDeploymentIdentity::new(
        digest(&report.network_id).ok_or(Refusal::RunReportGrammar)?,
        digest(&report.genesis_id).ok_or(Refusal::RunReportGrammar)?,
    )
    .map_err(|_| Refusal::RunReportGrammar)?;
    let evidence = OperatorEvidence::replay_recorded(identity, &payload.exchanges)?;
    verify_transaction(&payload.exchanges)?;
    compare_rows(&evidence, &payload)?;
    Ok(ValidatedNativeOperatorCorpus {
        evidence,
        report,
        capture_suite,
        capabilities,
        exchanges: payload.exchanges,
        recorded_message: payload.message,
    })
}

fn validate_inputs(
    files: &[ArchiveFile<'_>],
    sizes: &[usize; 4],
    manifest: &str,
    address: &str,
) -> ImportResult<ValidatedNativeOperatorCorpus> {
    validate_census(files, sizes)?;
    validate_manifest(files, manifest)?;
    let report = parse_report(files[3].bytes, address)?;
    binding("manifest-sha256", report.manifest_digest == manifest)?;
    let mut timing = Cursor::new("operator timing", files[1].bytes)?;
    timing.exact("timing-schema 1")?;
    timing.exact("ceremony-id report")?;
    timing.exact("status passed")?;
    timing.done()?;
    admit_capture(files[0].bytes, report)
}

/// Admits the immutable operator archive once through grammar and planner checks.
///
/// # Errors
/// Returns the refusal naming the first failed admission layer.
pub fn run_of_record() -> Result<&'static ValidatedNativeOperatorCorpus, NativeOperatorImportRefusal>
{
    static CORPUS: OnceLock<ImportResult<ValidatedNativeOperatorCorpus>> = OnceLock::new();
    match CORPUS.get_or_init(|| {
        let corpus = validate_inputs(
            &FILES,
            &FILE_SIZES,
            NATIVE_OPERATOR_MANIFEST_SHA256,
            NATIVE_OPERATOR_RUN_ADDRESS,
        )?;
        binding(
            "pinned-ceremony-digest",
            corpus.report.ceremony_digest == CEREMONY_SHA256,
        )?;
        Ok(corpus)
    }) {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct OwnedCorpus {
        bytes: [Vec<u8>; 4],
    }

    impl OwnedCorpus {
        fn new() -> Self {
            Self {
                bytes: FILES.map(|file| file.bytes.to_vec()),
            }
        }

        fn inputs(&self) -> [ArchiveFile<'_>; 4] {
            std::array::from_fn(|index| ArchiveFile {
                name: FILES[index].name,
                bytes: &self.bytes[index],
            })
        }

        fn pinned(&self) -> ImportResult<ValidatedNativeOperatorCorpus> {
            validate_inputs(
                &self.inputs(),
                &FILE_SIZES,
                NATIVE_OPERATOR_MANIFEST_SHA256,
                NATIVE_OPERATOR_RUN_ADDRESS,
            )
        }

        fn replace(&mut self, index: usize, from: &str, to: &str) {
            let text = std::str::from_utf8(&self.bytes[index]).expect("text fixture");
            assert!(text.contains(from), "missing mutation operand");
            self.bytes[index] = text.replacen(from, to, 1).into_bytes();
        }

        fn rebind(&mut self, content: bool) -> ImportResult<ValidatedNativeOperatorCorpus> {
            if content {
                let text = std::str::from_utf8(&self.bytes[0]).expect("capture");
                let position = text.find("capture-content-sha256 ").expect("content hash");
                let hash = hex_bytes(&tagged::sha256(&self.bytes[0][..position]));
                let start = position + "capture-content-sha256 ".len();
                self.bytes[0][start..start + 64].copy_from_slice(hash.as_bytes());
            }
            let capture_hash = hex_bytes(&tagged::sha256(&self.bytes[0]));
            let mut manifest = String::new();
            for (file, bytes) in FILES.iter().zip(&self.bytes).take(2) {
                let _ = writeln!(
                    manifest,
                    "{}  {}",
                    hex_bytes(&tagged::sha256(bytes)),
                    file.name
                );
            }
            self.bytes[2] = manifest.into_bytes();
            let manifest_hash = hex_bytes(&tagged::sha256(&self.bytes[2]));
            let report = std::str::from_utf8(&self.bytes[3]).expect("report");
            let mut rebound = String::new();
            for line in report.lines() {
                if line.starts_with("ceremony report ") {
                    let _ = writeln!(rebound, "ceremony report {capture_hash}");
                } else if line.starts_with("manifest-sha256 ") {
                    let _ = writeln!(rebound, "manifest-sha256 {manifest_hash}");
                } else {
                    let _ = writeln!(rebound, "{line}");
                }
            }
            self.bytes[3] = rebound.into_bytes();
            let address = hex_bytes(&tagged::sha256(&self.bytes[3]));
            let sizes = std::array::from_fn(|index| self.bytes[index].len());
            validate_inputs(&self.inputs(), &sizes, &manifest_hash, &address)
        }

        fn payload_replace(&mut self, from: &str, to: &str) {
            let text = std::str::from_utf8(&self.bytes[0]).expect("capture");
            let line = text
                .lines()
                .find(|line| line.starts_with("legacy-rendering "))
                .expect("payload")
                .to_owned();
            let encoded = line.split(' ').nth(2).expect("hex");
            let payload = String::from_utf8(decode_hex(encoded).expect("bytes")).expect("text");
            assert!(payload.contains(from));
            let changed = payload.replacen(from, to, 1);
            self.replace(
                0,
                &line,
                &format!(
                    "legacy-rendering {} {}",
                    changed.len(),
                    hex_bytes(changed.as_bytes())
                ),
            );
        }
    }

    #[test]
    fn every_file_byte_flip_stops_at_its_outer_layer() {
        for index in 0..4 {
            let mut corpus = OwnedCorpus::new();
            corpus.bytes[index][0] ^= 1;
            let refusal = corpus.pinned().expect_err("corrupt file");
            match index {
                0 | 1 => assert_eq!(
                    refusal,
                    Refusal::ManifestDigest {
                        name: FILES[index].name.to_owned()
                    }
                ),
                2 => assert_eq!(refusal, Refusal::ManifestHash),
                3 => assert_eq!(refusal, Refusal::RunReportGrammar),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn census_and_lengths_are_fixed() {
        assert_eq!(
            validate_census(&FILES[..3], &FILE_SIZES),
            Err(Refusal::CorpusCensus)
        );
        let mut files = FILES;
        files.swap(0, 1);
        assert_eq!(
            validate_census(&files, &FILE_SIZES),
            Err(Refusal::CorpusCensus)
        );
        let mut corpus = OwnedCorpus::new();
        corpus.bytes[0].pop();
        assert!(matches!(corpus.pinned(), Err(Refusal::FileSize { .. })));
    }

    #[test]
    fn manifest_grammar_rejects_extra_and_unsorted_lines() {
        for malformed in [
            format!(
                "{}extra\n",
                std::str::from_utf8(FILES[2].bytes).expect("manifest")
            ),
            std::str::from_utf8(FILES[2].bytes)
                .expect("manifest")
                .lines()
                .rev()
                .fold(String::new(), |mut text, line| {
                    let _ = writeln!(text, "{line}");
                    text
                }),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.bytes[2] = malformed.into_bytes();
            assert_eq!(
                validate_manifest(
                    &corpus.inputs(),
                    &hex_bytes(&tagged::sha256(&corpus.bytes[2]))
                ),
                Err(Refusal::ManifestGrammar)
            );
        }
    }

    #[test]
    fn valid_grammar_still_requires_the_pinned_report_address() {
        let mut corpus = OwnedCorpus::new();
        let text = std::str::from_utf8(&corpus.bytes[3]).expect("report");
        let position = text.find("suite-tree ").expect("tree") + "suite-tree ".len();
        corpus.bytes[3][position] = b'2';
        assert_eq!(corpus.pinned(), Err(Refusal::RunReportAddress));
    }

    #[test]
    fn capture_content_hash_is_independent_of_manifest_rebinding() {
        let mut corpus = OwnedCorpus::new();
        corpus.payload_replace(
            "state-construction incomplete",
            "state-construction incompletE",
        );
        assert_eq!(corpus.rebind(false), Err(Refusal::CaptureContentHash));
    }

    #[test]
    fn report_and_capture_suite_bindings_must_agree() {
        let mut corpus = OwnedCorpus::new();
        let text = std::str::from_utf8(&corpus.bytes[0]).expect("capture");
        let start = text.find("suite-commit ").expect("commit") + "suite-commit ".len();
        corpus.bytes[0][start] = b'4';
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::CrossFileBinding {
                field: "suite-commit"
            })
        );
    }

    #[test]
    fn response_links_are_checked_before_interpretation() {
        let mut corpus = OwnedCorpus::new();
        corpus.replace(
            0,
            "response-request-id 9 726571756573742d30",
            "response-request-id 9 726571756573742d31",
        );
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::IdentifierLink {
                step: NATIVE_STEPS[0]
            })
        );
    }

    #[test]
    fn protocol_and_environment_rosters_are_closed() {
        let mut corpus = OwnedCorpus::new();
        corpus.replace(
            0,
            "handshake-protocol-schema 8",
            "handshake-protocol-schema 7",
        );
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::CrossFileBinding {
                field: "protocol-revision"
            })
        );
        let mut corpus = OwnedCorpus::new();
        corpus.replace(
            0,
            "environment-capability-count 14",
            "environment-capability-count 13",
        );
        assert!(matches!(
            corpus.rebind(true),
            Err(Refusal::TranscriptGrammar { .. })
        ));
    }

    #[test]
    fn timing_status_has_its_own_closed_grammar() {
        let mut corpus = OwnedCorpus::new();
        corpus.bytes[1][0] ^= 1;
        assert!(
            matches!(corpus.rebind(true), Err(Refusal::TranscriptGrammar { name, .. }) if name == "operator timing")
        );
    }

    #[test]
    fn recorded_json_revision_is_checked_independently() {
        let mut corpus = OwnedCorpus::new();
        corpus.payload_replace("7b22736368656d61223a38", "7b22736368656d61223a37");
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::ResponseGrammar {
                step: NATIVE_STEPS[0]
            })
        );
    }

    #[test]
    fn payload_step_order_and_signing_census_are_closed() {
        for (from, to) in [
            (
                "operator-step sign-operator",
                "operator-step sign-wrong-key",
            ),
            ("spent-output-count 1", "spent-output-count 2"),
            (
                "state-construction incomplete",
                "state-construction complete",
            ),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.payload_replace(from, to);
            assert!(matches!(
                corpus.rebind(true),
                Err(Refusal::TranscriptGrammar { .. })
            ));
        }
    }

    #[test]
    fn recorded_rows_cannot_override_derived_standings() {
        let mut corpus = OwnedCorpus::new();
        corpus.payload_replace(
            "row operator-positive expected Some(Accepted)",
            "row operator-positive expected None",
        );
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::RowStanding {
                row: "operator-positive"
            })
        );
    }

    #[test]
    fn recorded_message_cannot_override_recomputation() {
        let mut corpus = OwnedCorpus::new();
        let message = hex_bytes(run_of_record().expect("corpus").recorded_message());
        let replacement = format!(
            "{}{}",
            if message.starts_with('0') { "1" } else { "0" },
            &message[1..]
        );
        corpus.payload_replace(
            &format!("recomputed-message 32 {message}"),
            &format!("recomputed-message 32 {replacement}"),
        );
        assert_eq!(corpus.rebind(true), Err(Refusal::RecomputedMessage));
    }

    fn replay(
        exchanges: &[(OperationSubject, NativeOperationResponse)],
    ) -> ImportResult<OperatorEvidence> {
        let corpus = run_of_record().expect("corpus");
        OperatorEvidence::replay_recorded(
            CandidateDeploymentIdentity::new(
                digest(corpus.report().network_id()).expect("network"),
                digest(corpus.report().genesis_id()).expect("genesis"),
            )
            .expect("deployment"),
            exchanges,
        )
    }

    #[test]
    fn settlement_independently_refuses_each_corrupted_signature() {
        for index in [2, 3] {
            let mut exchanges = run_of_record().expect("corpus").exchanges().to_vec();
            exchanges[index].1.script_path_witness[0][0] ^= 1;
            assert_eq!(
                replay(&exchanges),
                Err(Refusal::ReplaySettlement {
                    step: NATIVE_STEPS[index]
                })
            );
        }
    }

    #[test]
    fn settlement_refuses_changed_readback_bytes() {
        let mut exchanges = run_of_record().expect("corpus").exchanges().to_vec();
        exchanges[10]
            .1
            .mined_readback
            .as_mut()
            .expect("readback")
            .raw_transaction[0] ^= 1;
        assert_eq!(
            replay(&exchanges),
            Err(Refusal::ReplaySettlement {
                step: NATIVE_STEPS[10]
            })
        );
    }

    #[test]
    fn replay_refuses_changed_submission_signing_and_funding_subjects() {
        for index in [0, 1, 2, 3, 4, 10] {
            let mut exchanges = run_of_record().expect("corpus").exchanges().to_vec();
            match &mut exchanges[index].0 {
                OperationSubject::Funding(subject) => subject.amount_per_output += 1,
                OperationSubject::ScriptPathSigning(subject) => {
                    subject.executing_leaf.script[0] ^= 1;
                }
                OperationSubject::Submission(subject) => subject.transaction_bytes[0] ^= 1,
                _ => panic!("unexpected subject"),
            }
            assert_eq!(
                replay(&exchanges),
                Err(Refusal::ReplaySubject {
                    step: NATIVE_STEPS[index]
                })
            );
        }
    }

    #[test]
    fn accepted_identity_is_recomputed_from_the_readback() {
        let mut exchanges = run_of_record().expect("corpus").exchanges().to_vec();
        let response = &mut exchanges[10].1;
        response.accepted_txid = Some("00".repeat(32));
        response
            .mined_readback
            .as_mut()
            .expect("readback")
            .transaction_id = "00".repeat(32);
        assert_eq!(
            verify_transaction(&exchanges),
            Err(Refusal::TransactionIdentity)
        );
    }
}
