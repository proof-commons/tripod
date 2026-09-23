//! Pinned four-file admission and planner replay of maturity announcement runs.
//!
//! Names, sizes and addresses bind each archive before its closed grammars are interpreted. The historical refused loader is one set of pins; the second immutable corpus binds a variable-metadata acceptance with its own schedule and sponsorless expectation. Each corpus also pins the cargo path, framework revision mode and funding advertisement of the host that ran it.
//!
//! An accepted payload carries mined readback beside the capture's request bytes, verdict, layer, target identity and detail. Replay checks the reconstructed response against the submitted transaction. This establishes transcript consistency under the schedule, not executor provenance, current-root freshness or a promoted standing.

use std::fmt::Write as _;
use std::sync::OnceLock;

use linker::CandidateDeploymentIdentity;
use target_elements_conformance::constructor::tagged;
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::{
    MinedFundingReadback, NativeOperationResponse, ObservedOutcomeLayer, OperationSubject,
    TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::bytes::Txid;
use transaction::operator_right::BranchContext;

use crate::maturity_closure::MaturityWitnessSelection;
use crate::maturity_native::{
    ANNOUNCEMENT_STEPS, MaturityAcceptanceObligation, MaturityAcceptanceRoute,
    MaturityNativeEvidence, MaturityNativePlanRefusal, MaturityNativeStanding,
};

/// The archive's witness schedule, established by exact replay of all three
/// recorded requests rather than selected as a deployment preference.
pub const MATURITY_RUN_SCHEDULE: tapscript::StateWitnessSchedule =
    tapscript::StateWitnessSchedule::WholeMetadata;

/// SHA-256 of the exact two-entry evidence manifest.
pub const MATURITY_MANIFEST_SHA256: &str =
    "66578a5b6e5aa150827c8e5c20a55003a0a21ddfbc1ba08b48943bd26d57b7e1";
/// SHA-256 of the report binding the complete maturity run.
pub const MATURITY_RUN_ADDRESS: &str =
    "fb2912ecb8e8b3a6ba8af7839167e644f6398fe7a296f89ff9fc757be83b31f3";

/// The witness schedule established by replay of the accepted archive.
pub const MATURITY_VARIABLE_RUN_SCHEDULE: tapscript::StateWitnessSchedule =
    tapscript::StateWitnessSchedule::VariableMetadata;
/// SHA-256 of the accepted archive's exact two-entry manifest.
pub const MATURITY_VARIABLE_MANIFEST_SHA256: &str =
    "a59b750c459c95d0b72fe2e77c45c6e959c1407c702ce13b9e4a3ba0c487dece";
/// SHA-256 of the accepted archive's report.
pub const MATURITY_VARIABLE_RUN_ADDRESS: &str =
    "b624e245fcdfe839ff9bb3e94285165ab0c9fbdd606748fd115ae8b02dc87b0c";

const CEREMONY_SHA256: &str = "d48e105b85708cd8edadbe423ed020c48fdbe1c0225f67724ff6f68449d25515";

/// One named file supplied to the four-file maturity corpus admission.
#[derive(Clone, Copy)]
pub struct MaturityArchiveFile<'a> {
    name: &'a str,
    bytes: &'a [u8],
}

impl<'a> MaturityArchiveFile<'a> {
    /// Binds a file name to its exact bytes for admission.
    #[must_use]
    pub const fn new(name: &'a str, bytes: &'a [u8]) -> Self {
        Self { name, bytes }
    }

    /// The name used by the ordered census and manifest.
    #[must_use]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// The exact bytes to admit.
    #[must_use]
    pub const fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
}

const FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile {
        name: "3a690139.report.capture",
        bytes: include_bytes!("../fixtures/maturity-run-of-record/3a690139.report.capture"),
    },
    MaturityArchiveFile {
        name: "3a690139.report.capture.timing",
        bytes: include_bytes!("../fixtures/maturity-run-of-record/3a690139.report.capture.timing"),
    },
    MaturityArchiveFile {
        name: "MANIFEST.sha256",
        bytes: include_bytes!("../fixtures/maturity-run-of-record/MANIFEST.sha256"),
    },
    MaturityArchiveFile {
        name: "RUN-REPORT",
        bytes: include_bytes!("../fixtures/maturity-run-of-record/RUN-REPORT"),
    },
];
const FILE_SIZES: [usize; 4] = [31_153, 49, 187, 1_927];

const VARIABLE_FILES: [MaturityArchiveFile<'static>; 4] = [
    MaturityArchiveFile {
        name: "cae3bd7c.report.capture",
        bytes: include_bytes!(
            "../fixtures/maturity-variable-run-of-record/cae3bd7c.report.capture"
        ),
    },
    MaturityArchiveFile {
        name: "cae3bd7c.report.capture.timing",
        bytes: include_bytes!(
            "../fixtures/maturity-variable-run-of-record/cae3bd7c.report.capture.timing"
        ),
    },
    MaturityArchiveFile {
        name: "MANIFEST.sha256",
        bytes: include_bytes!("../fixtures/maturity-variable-run-of-record/MANIFEST.sha256"),
    },
    MaturityArchiveFile {
        name: "RUN-REPORT",
        bytes: include_bytes!("../fixtures/maturity-variable-run-of-record/RUN-REPORT"),
    },
];

/// The expected outcome of the sponsorless operation in a pinned corpus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturitySponsorlessExpectation {
    /// A relay refusal with its exact recorded response detail.
    RelayRefusal { detail: &'static str },
    /// An accepted submission with a mined readback carried in the payload.
    Acceptance,
}

/// How the capture binds the framework revision to its run report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityFrameworkRevision {
    /// The framework revision was recorded as empty text.
    Unrecorded,
    /// The framework revision equals the report's intended executed tip.
    IntendedTip,
}

/// Source of a disclosed environment premise that corpus admission does not check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityPremiseProvenance {
    /// The node arguments passed by the native executor.
    ExecutorArguments,
}

/// Fee floors disclosed for the node that ran an admitted archive.
///
/// The immutable RUN-REPORT grammar carries no fee line, so archive admission
/// cannot verify these values. They are declared from the executor's node
/// arguments and remain a premise rather than a checked binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturityDisclosedFeeFloors {
    min_relay_tx_fee: &'static str,
    block_min_tx_fee: &'static str,
    provenance: MaturityPremiseProvenance,
}

impl MaturityDisclosedFeeFloors {
    /// Declares the two executor node arguments without attributing them to archive bytes.
    #[must_use]
    pub const fn declared(min_relay_tx_fee: &'static str, block_min_tx_fee: &'static str) -> Self {
        Self {
            min_relay_tx_fee,
            block_min_tx_fee,
            provenance: MaturityPremiseProvenance::ExecutorArguments,
        }
    }

    /// The declared minimum relay transaction fee argument.
    #[must_use]
    pub const fn min_relay_tx_fee(&self) -> &'static str {
        self.min_relay_tx_fee
    }

    /// The declared minimum block transaction fee argument.
    #[must_use]
    pub const fn block_min_tx_fee(&self) -> &'static str {
        self.block_min_tx_fee
    }

    /// Where the two declared values came from.
    #[must_use]
    pub const fn provenance(&self) -> MaturityPremiseProvenance {
        self.provenance
    }
}

/// The fee arguments disclosed for both admitted runs of the native executor.
pub const MATURITY_DISCLOSED_FEE_FLOORS: MaturityDisclosedFeeFloors =
    MaturityDisclosedFeeFloors::declared("0", "0");

/// The addresses and declaration that bind one immutable four-file corpus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturityCorpusPins<'a> {
    schedule: tapscript::StateWitnessSchedule,
    names: [&'a str; 4],
    sizes: [usize; 4],
    manifest_sha256: &'a str,
    run_address: &'a str,
    ceremony_sha256: &'a str,
    sponsorless: MaturitySponsorlessExpectation,
    cargo_path: &'a str,
    framework_revision: MaturityFrameworkRevision,
    funding_count: usize,
    fee_floors: MaturityDisclosedFeeFloors,
}

impl<'a> MaturityCorpusPins<'a> {
    /// Collects the ordered file pins, addresses, sponsorless declaration and host premises.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "all eight pins are declared at every call site rather than defaulted"
    )]
    pub const fn new(
        schedule: tapscript::StateWitnessSchedule,
        files: ([&'a str; 4], [usize; 4]),
        addresses: (&'a str, &'a str, &'a str),
        sponsorless: MaturitySponsorlessExpectation,
        cargo_path: &'a str,
        framework_revision: MaturityFrameworkRevision,
        funding_count: usize,
        fee_floors: MaturityDisclosedFeeFloors,
    ) -> Self {
        let (names, sizes) = files;
        let (manifest_sha256, run_address, ceremony_sha256) = addresses;
        Self {
            schedule,
            names,
            sizes,
            manifest_sha256,
            run_address,
            ceremony_sha256,
            sponsorless,
            cargo_path,
            framework_revision,
            funding_count,
            fee_floors,
        }
    }

    /// The schedule used for exact request replay.
    #[must_use]
    pub const fn schedule(&self) -> tapscript::StateWitnessSchedule {
        self.schedule
    }

    /// The ordered file names.
    #[must_use]
    pub const fn names(&self) -> &[&'a str; 4] {
        &self.names
    }

    /// The ordered file lengths.
    #[must_use]
    pub const fn sizes(&self) -> &[usize; 4] {
        &self.sizes
    }

    /// SHA-256 of the manifest bytes.
    #[must_use]
    pub const fn manifest_sha256(&self) -> &'a str {
        self.manifest_sha256
    }

    /// SHA-256 of the report bytes.
    #[must_use]
    pub const fn run_address(&self) -> &'a str {
        self.run_address
    }

    /// SHA-256 of the capture bytes.
    #[must_use]
    pub const fn ceremony_sha256(&self) -> &'a str {
        self.ceremony_sha256
    }

    /// The declared sponsorless outcome grammar.
    #[must_use]
    pub const fn sponsorless(&self) -> MaturitySponsorlessExpectation {
        self.sponsorless
    }

    /// The executable path recorded at the start of the report's cargo argv.
    #[must_use]
    pub const fn cargo_path(&self) -> &'a str {
        self.cargo_path
    }

    /// The framework revision binding used by the capture handshake.
    #[must_use]
    pub const fn framework_revision(&self) -> MaturityFrameworkRevision {
        self.framework_revision
    }

    /// The number of funding advertisements in the capture environment.
    #[must_use]
    pub const fn funding_count(&self) -> usize {
        self.funding_count
    }

    /// The declared fee floors under which the executor ran this archive.
    #[must_use]
    pub const fn fee_floors(&self) -> MaturityDisclosedFeeFloors {
        self.fee_floors
    }
}

const HISTORICAL_PINS: MaturityCorpusPins<'static> = MaturityCorpusPins::new(
    MATURITY_RUN_SCHEDULE,
    (
        [
            "3a690139.report.capture",
            "3a690139.report.capture.timing",
            "MANIFEST.sha256",
            "RUN-REPORT",
        ],
        FILE_SIZES,
    ),
    (
        MATURITY_MANIFEST_SHA256,
        MATURITY_RUN_ADDRESS,
        CEREMONY_SHA256,
    ),
    MaturitySponsorlessExpectation::RelayRefusal {
        detail: "bad-witness-nonstandard",
    },
    "/workspace/toolchains/cargo/bin/cargo",
    MaturityFrameworkRevision::Unrecorded,
    0,
    MATURITY_DISCLOSED_FEE_FLOORS,
);

const VARIABLE_PINS: MaturityCorpusPins<'static> = MaturityCorpusPins::new(
    MATURITY_VARIABLE_RUN_SCHEDULE,
    (
        [
            "cae3bd7c.report.capture",
            "cae3bd7c.report.capture.timing",
            "MANIFEST.sha256",
            "RUN-REPORT",
        ],
        [55_271, 49, 187, 1_927],
    ),
    (
        MATURITY_VARIABLE_MANIFEST_SHA256,
        MATURITY_VARIABLE_RUN_ADDRESS,
        "fa474763a165c60d4e48b7c0c06f01ce79204a1fc4f9020bd7612c82e22bb3ee",
    ),
    MaturitySponsorlessExpectation::Acceptance,
    "/workspace/toolchains/cargo/bin/cargo",
    MaturityFrameworkRevision::IntendedTip,
    0,
    MATURITY_DISCLOSED_FEE_FLOORS,
);

/// The layer at which maturity corpus admission stopped.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MaturityCorpusImportRefusal {
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
    /// The payload contradicts evidence independently derived by planner replay.
    PayloadDisagreement { field: &'static str },
    /// The planner refused construction, an exact request, or a recorded response.
    Derivation(MaturityNativePlanRefusal),
}

type ImportResult<T> = Result<T, MaturityCorpusImportRefusal>;
use MaturityCorpusImportRefusal as Refusal;

/// Report facts admitted together with the complete corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityReportFacts {
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

impl MaturityReportFacts {
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

/// Complete maturity evidence from admitted bytes with declared executor fee floors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMaturityCorpus {
    evidence: MaturityNativeEvidence,
    report: MaturityReportFacts,
    fee_floors: MaturityDisclosedFeeFloors,
    capture_suite: [String; 2],
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    capabilities: Vec<String>,
    recorded_refusal_detail: String,
}

impl ValidatedMaturityCorpus {
    /// The composed schedule established by replay of the admitted exchanges.
    #[must_use]
    pub const fn schedule(&self) -> tapscript::StateWitnessSchedule {
        self.evidence.schedule()
    }

    /// The evidence produced by the planner's reviewed non-mock settlement.
    #[must_use]
    pub const fn evidence(&self) -> &MaturityNativeEvidence {
        &self.evidence
    }

    /// The fee floors declared for this admitted archive's executor environment.
    #[must_use]
    pub const fn fee_floors(&self) -> MaturityDisclosedFeeFloors {
        self.fee_floors
    }

    /// The closed report's cross-checked facts.
    #[must_use]
    pub const fn report(&self) -> &MaturityReportFacts {
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
    pub fn exchanges(&self) -> &[(OperationStep, NativeOperationResponse)] {
        &self.exchanges
    }

    /// The exact ordered environment capability roster.
    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    /// The sponsorless operation's recorded response detail: a refusal reason or empty on acceptance.
    #[must_use]
    pub fn recorded_refusal_detail(&self) -> &str {
        &self.recorded_refusal_detail
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

fn validate_census(
    files: &[MaturityArchiveFile<'_>],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<()> {
    if files.len() != pins.names.len()
        || files
            .iter()
            .zip(pins.names)
            .any(|(file, name)| file.name != name)
    {
        return Err(Refusal::CorpusCensus);
    }
    for (file, expected) in files.iter().zip(&pins.sizes) {
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

fn validate_manifest(files: &[MaturityArchiveFile<'_>], expected_hash: &str) -> ImportResult<()> {
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

fn parse_report_suite(
    cursor: &mut Cursor<'_>,
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<[String; 5]> {
    cursor.exact("run-report-schema native-maturity-run-report 1")?;
    cursor.exact("capture-format-schema native-maturity-capture 1")?;
    let suite_commit = cursor.value("suite-commit")?.to_owned();
    let suite_tree = cursor.value("suite-tree")?.to_owned();
    if !source_identity(&suite_commit) || !source_identity(&suite_tree) {
        return Err(cursor.refusal());
    }
    cursor.exact("suite-clean yes")?;
    cursor.exact("rust-test-target maturity_native")?;
    if cursor.text("cargo-argv")?
        != format!(
            "{} test -p tripod-vectors --test maturity_native -- --ignored --test-threads=1",
            pins.cargo_path()
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

fn parse_report_body(
    bytes: &[u8],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<MaturityReportFacts> {
    let mut cursor = Cursor::new("RUN-REPORT", bytes)?;
    let [
        suite_commit,
        suite_tree,
        expected_tip,
        binary_revision,
        intended_tip,
    ] = parse_report_suite(&mut cursor, pins)?;
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
    Ok(MaturityReportFacts {
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

fn parse_report(bytes: &[u8], pins: &MaturityCorpusPins<'_>) -> ImportResult<MaturityReportFacts> {
    let report = parse_report_body(bytes, pins).map_err(|_| Refusal::RunReportGrammar)?;
    if report.run_address != pins.run_address() {
        return Err(Refusal::RunReportAddress);
    }
    Ok(report)
}

const CAPABILITIES: [&str; 12] = [
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
];

fn parse_environment(
    cursor: &mut Cursor<'_>,
    report: &MaturityReportFacts,
    pins: &MaturityCorpusPins<'_>,
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
        "environment-capability-count 12",
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
    cursor.exact(&format!(
        "environment-funding-count {}",
        pins.funding_count()
    ))?;
    for _ in 0..pins.funding_count() {
        cursor.value("environment-funding")?;
    }
    Ok(capabilities)
}

fn parse_header(
    cursor: &mut Cursor<'_>,
    report: &MaturityReportFacts,
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<[String; 2]> {
    cursor.exact("native-capture-schema 2")?;
    cursor.exact("ceremony-id report")?;
    if cursor.text("rust-test-name")? != "the_maturity_announcement_runs_against_a_real_target" {
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
    let framework_revision = match pins.framework_revision() {
        MaturityFrameworkRevision::Unrecorded => "",
        MaturityFrameworkRevision::IntendedTip => report.intended_tip.as_str(),
    };
    for (field, expected) in [
        ("handshake-adapter-name", report.adapter_name.as_str()),
        ("handshake-adapter-version", report.adapter_version.as_str()),
        ("handshake-framework-revision", framework_revision),
        ("handshake-node-name", report.node_name.as_str()),
        ("handshake-node-version", report.node_version.as_str()),
        (
            "handshake-binary-reported-revision",
            report.binary_revision.as_str(),
        ),
        (
            "handshake-intended-executed-tip",
            report.intended_tip.as_str(),
        ),
    ] {
        binding(field, cursor.text(field)? == expected)?;
    }
    cursor.exact("handshake-upstream-base 0 ")?;
    cursor.exact("handshake-topic-count 0")?;
    Ok([suite_commit, suite_tree])
}

struct RecordedOperation {
    step: OperationStep,
    layer: ObservedOutcomeLayer,
    detail: String,
    accepted_identity: Option<Txid>,
    request_bytes: Vec<u8>,
}

// These are the two exact JSON shapes emitted by this capture's serializer.
// Fixed punctuation and canonical numbers close the grammar without admitting
// duplicate, unknown, reordered or omitted members through a second JSON model.
fn json_bytes(text: &str) -> Option<Vec<u8>> {
    let body = text.strip_prefix('[')?.strip_suffix(']')?;
    if body.is_empty() {
        return Some(Vec::new());
    }
    body.split(',')
        .map(|value| number(value).and_then(|value| u8::try_from(value).ok()))
        .collect()
}

fn parse_subject(text: &str, index: usize) -> Option<OperationSubject> {
    if index == 2 {
        let array = text
            .strip_prefix("{\"transaction_bytes\":")?
            .strip_suffix('}')?;
        return Some(OperationSubject::Submission(Box::new(
            TargetSubmissionSubject {
                transaction_bytes: json_bytes(array)?,
            },
        )));
    }
    let rest = text.strip_prefix("{\"issue_asset\":")?;
    let (issue, rest) = rest.split_once(",\"asset\":")?;
    let issue_asset = match issue {
        "true" => true,
        "false" => false,
        _ => return None,
    };
    let (asset, rest) = rest.split_once(",\"output_program\":")?;
    let asset = if asset == "null" {
        None
    } else {
        let asset = asset.strip_prefix('"')?.strip_suffix('"')?;
        digest(asset)?;
        Some(asset.to_owned())
    };
    let (program, rest) = rest.split_once(",\"outputs\":")?;
    let (outputs, amount) = rest.split_once(",\"amount_per_output\":")?;
    let amount = amount.strip_suffix('}')?;
    number(amount)?;
    Some(OperationSubject::Funding(Box::new(TargetFundingSubject {
        issue_asset,
        asset,
        output_program: json_bytes(program)?,
        outputs: u8::try_from(number(outputs)?).ok()?,
        amount_per_output: amount.parse().ok()?,
    })))
}

fn parse_operation(
    cursor: &mut Cursor<'_>,
    index: usize,
    name: &'static str,
    expectation: MaturitySponsorlessExpectation,
) -> ImportResult<RecordedOperation> {
    cursor.exact(&format!("operation {index} begin"))?;
    for (field, expected) in [
        ("operation-id", format!("operation-{index}")),
        ("request-id", format!("request-{index}")),
    ] {
        binding(field, cursor.text(field)? == expected)?;
    }
    cursor.exact("request-role auxiliary")?;
    let request_bytes = cursor.bytes("request-bytes")?;
    let subject =
        parse_subject(&cursor.text("request-subject")?, index).ok_or_else(|| cursor.refusal())?;
    binding(
        "request-bytes",
        match &subject {
            OperationSubject::Funding(_) => request_bytes.is_empty(),
            OperationSubject::Submission(submission) => {
                !request_bytes.is_empty() && request_bytes == submission.transaction_bytes
            }
            _ => false,
        },
    )?;
    for (field, expected) in [
        ("response-id", format!("response-{index}")),
        ("response-request-id", format!("request-{index}")),
        ("response-operation-id", format!("operation-{index}")),
    ] {
        binding(field, cursor.text(field)? == expected)?;
    }
    let accepted = index < 2 || expectation == MaturitySponsorlessExpectation::Acceptance;
    cursor.exact(if accepted {
        "response-verdict accepted"
    } else {
        "response-verdict refused"
    })?;
    cursor.exact(if accepted {
        "response-layer accepted"
    } else {
        "response-layer relay-policy-rejection"
    })?;
    let accepted_identity = if index == 2 && accepted {
        Some(
            Txid::from_target_display(cursor.value("response-target-identity")?)
                .map_err(|_| cursor.refusal())?,
        )
    } else {
        cursor.exact("response-target-identity none")?;
        None
    };
    let detail = cursor.text("response-detail")?;
    if detail
        != if accepted {
            ""
        } else {
            match expectation {
                MaturitySponsorlessExpectation::RelayRefusal { detail } => detail,
                MaturitySponsorlessExpectation::Acceptance => "",
            }
        }
    {
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
        step: OperationStep::new(name, subject),
        detail,
        accepted_identity,
        request_bytes,
        layer: if accepted {
            ObservedOutcomeLayer::Accepted
        } else {
            ObservedOutcomeLayer::RelayPolicyRejection
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

enum PayloadAcceptance {
    Outstanding {
        claim: String,
        routes: [String; 2],
    },
    Established {
        schedule: tapscript::StateWitnessSchedule,
        identity: Txid,
        witness_identity: Txid,
        block_hash: [u8; 32],
        block_height: u32,
    },
}

struct Payload {
    identity: CandidateDeploymentIdentity,
    branch: BranchContext,
    exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    rows: Vec<String>,
    standing: String,
    acceptance: PayloadAcceptance,
}

fn parse_acceptance(
    cursor: &mut Cursor<'_>,
    operations: &[RecordedOperation],
    exchanges: &mut [(OperationStep, NativeOperationResponse)],
    expectation: MaturitySponsorlessExpectation,
) -> ImportResult<PayloadAcceptance> {
    let claim = cursor.value("sponsorless-acceptance")?.to_owned();
    match expectation {
        MaturitySponsorlessExpectation::RelayRefusal { .. } => {
            if claim == "established" {
                return Err(cursor.refusal());
            }
            Ok(PayloadAcceptance::Outstanding {
                claim,
                routes: [
                    cursor.value("acceptance-route")?.to_owned(),
                    cursor.value("acceptance-route")?.to_owned(),
                ],
            })
        }
        MaturitySponsorlessExpectation::Acceptance => {
            if claim != "established" {
                return Err(cursor.refusal());
            }
            let schedule = match cursor.value("accepted-schedule")? {
                "whole-metadata" => tapscript::StateWitnessSchedule::WholeMetadata,
                "variable-metadata" => tapscript::StateWitnessSchedule::VariableMetadata,
                _ => return Err(cursor.refusal()),
            };
            let identity = Txid::from_target_display(cursor.value("accepted-identity")?)
                .map_err(|_| cursor.refusal())?;
            let witness_identity =
                Txid::from_target_display(cursor.value("accepted-witness-identity")?)
                    .map_err(|_| cursor.refusal())?;
            let block = cursor.value("accepted-block")?;
            let (block_hash, height) = block.split_once(' ').ok_or_else(|| cursor.refusal())?;
            let block_hash = digest(block_hash).ok_or_else(|| cursor.refusal())?;
            let block_height = number(height)
                .and_then(|height| u32::try_from(height).ok())
                .ok_or_else(|| cursor.refusal())?;
            let operation = operations.last().ok_or(Refusal::CorpusCensus)?;
            let capture_identity = operation
                .accepted_identity
                .ok_or_else(|| cursor.refusal())?;
            binding("accepted-identity", capture_identity == identity)?;
            let (_, response) = exchanges.last_mut().ok_or(Refusal::CorpusCensus)?;
            response.accepted_txid = Some(capture_identity.to_target_display());
            response.mined_readback = Some(MinedFundingReadback {
                transaction_id: capture_identity.to_target_display(),
                witness_transaction_id: witness_identity.to_target_display(),
                block_hash: hex_bytes(&block_hash),
                block_height,
                raw_transaction: operation.request_bytes.clone(),
            });
            Ok(PayloadAcceptance::Established {
                schedule,
                identity,
                witness_identity,
                block_hash,
                block_height,
            })
        }
    }
}

fn parse_payload(
    bytes: &[u8],
    operations: &[RecordedOperation],
    report: &MaturityReportFacts,
    expectation: MaturitySponsorlessExpectation,
) -> ImportResult<Payload> {
    let mut cursor = Cursor::new("maturity payload", bytes)?;
    cursor.exact("maturity-evidence-schema 1")?;
    let network = cursor.bytes("deployment-network-id")?;
    let genesis = cursor.bytes("deployment-genesis-id")?;
    binding(
        "payload-network-id",
        hex_bytes(&network) == report.network_id,
    )?;
    binding(
        "payload-genesis-id",
        hex_bytes(&genesis) == report.genesis_id,
    )?;
    let identity = CandidateDeploymentIdentity::new(
        network.try_into().map_err(|_| cursor.refusal())?,
        genesis.try_into().map_err(|_| cursor.refusal())?,
    )
    .map_err(|_| cursor.refusal())?;
    let identifier = cursor.bytes("branch-identifier")?;
    let checkpoint = cursor.value("branch-checkpoint")?;
    if identifier != [0x41; 32] || checkpoint != "7" {
        return Err(cursor.refusal());
    }
    let branch = BranchContext::new(
        identifier.try_into().map_err(|_| cursor.refusal())?,
        checkpoint.parse().map_err(|_| cursor.refusal())?,
    )
    .map_err(|_| cursor.refusal())?;
    cursor.exact("current-root-freshness unestablished")?;
    let mut exchanges = Vec::new();
    for (index, (name, operation)) in ANNOUNCEMENT_STEPS.iter().zip(operations).enumerate() {
        cursor.exact(&format!("maturity-step {name}"))?;
        let response =
            NativeOperationResponse::from_recorded_json(&cursor.bytes("validated-response")?)
                .map_err(|_| cursor.refusal())?;
        binding("response-case", response.case == *operation.step.case())?;
        binding(
            "response-summary",
            response.observed_layer == operation.layer
                && (index == 2 && expectation == MaturitySponsorlessExpectation::Acceptance
                    || response.accepted_txid.is_none())
                && response.observed_detail.as_deref().unwrap_or_default() == operation.detail,
        )?;
        exchanges.push((operation.step.clone(), response));
    }
    let rows = ANNOUNCEMENT_STEPS
        .iter()
        .map(|_| cursor.value("row").map(str::to_owned))
        .collect::<ImportResult<Vec<_>>>()?;
    let standing = cursor.value("sponsorless-standing")?.to_owned();
    let acceptance = parse_acceptance(&mut cursor, operations, &mut exchanges, expectation)?;
    cursor.done()?;
    Ok(Payload {
        identity,
        branch,
        exchanges,
        rows,
        standing,
        acceptance,
    })
}

const fn payload_agrees(field: &'static str, agrees: bool) -> ImportResult<()> {
    if agrees {
        Ok(())
    } else {
        Err(Refusal::PayloadDisagreement { field })
    }
}

fn compare_payload(
    evidence: &MaturityNativeEvidence,
    payload: &Payload,
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<()> {
    payload_agrees("deployment", evidence.identity() == &payload.identity)?;
    payload_agrees("branch", evidence.branch() == payload.branch)?;
    payload_agrees(
        "row-count",
        evidence.observations().len() == payload.rows.len(),
    )?;
    for (observation, recorded) in evidence.observations().iter().zip(&payload.rows) {
        let expected = format!(
            "{} declared {:?} observed {:?}",
            observation.subject(),
            observation.declared_layer(),
            observation.observed_layer()
        );
        payload_agrees(observation.subject(), *recorded == expected)?;
    }
    payload_agrees(
        "sponsorless-standing",
        evidence.standing() == MaturityNativeStanding::AnsweredAtDeclaredBoundary
            && payload.standing == format!("{:?}", evidence.standing()),
    )?;
    match (evidence.acceptance_obligation(), &payload.acceptance) {
        (
            MaturityAcceptanceObligation::Outstanding { routes },
            PayloadAcceptance::Outstanding {
                claim,
                routes: recorded,
            },
        ) => {
            payload_agrees("sponsorless-acceptance", claim == "outstanding")?;
            payload_agrees(
                "acceptance-routes",
                *routes
                    == [
                        MaturityAcceptanceRoute::RelayWitnessRestructure,
                        MaturityAcceptanceRoute::BlockLayerSubmissionSubject,
                    ]
                    && *recorded == routes.map(|route| format!("{route:?}")),
            )
        }
        (
            MaturityAcceptanceObligation::Established {
                schedule,
                identity,
                readback,
            },
            PayloadAcceptance::Established {
                schedule: recorded_schedule,
                identity: recorded_identity,
                witness_identity,
                block_hash,
                block_height,
            },
        ) => {
            payload_agrees(
                "accepted-schedule",
                *schedule == pins.schedule() && schedule == recorded_schedule,
            )?;
            payload_agrees(
                "accepted-identity",
                identity == recorded_identity && *identity == readback.identity(),
            )?;
            payload_agrees(
                "accepted-witness-identity",
                *witness_identity == readback.witness_identity(),
            )?;
            payload_agrees("accepted-block", block_hash == readback.block_hash())?;
            payload_agrees(
                "accepted-block-height",
                *block_height == readback.block_height(),
            )
        }
        _ => payload_agrees("sponsorless-acceptance", false),
    }
}

fn derive(
    payload: &Payload,
    schedule: tapscript::StateWitnessSchedule,
) -> ImportResult<MaturityNativeEvidence> {
    MaturityNativeEvidence::from_transcript(
        payload.identity.clone(),
        payload.branch,
        MaturityWitnessSelection::Retained(schedule),
        &payload.exchanges,
    )
    .map_err(Refusal::Derivation)
}

fn admit_capture(
    bytes: &[u8],
    report: MaturityReportFacts,
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<ValidatedMaturityCorpus> {
    binding(
        "ceremony-digest",
        hex_bytes(&tagged::sha256(bytes)) == report.ceremony_digest,
    )?;
    let mut cursor = Cursor::new("maturity capture", bytes)?;
    let capture_suite = parse_header(&mut cursor, &report, pins)?;
    let capabilities = parse_environment(&mut cursor, &report, pins)?;
    cursor.exact("digest-count 0")?;
    cursor.exact("operation-count 3")?;
    let operations = ANNOUNCEMENT_STEPS
        .iter()
        .enumerate()
        .map(|(index, name)| parse_operation(&mut cursor, index, name, pins.sponsorless()))
        .collect::<ImportResult<Vec<_>>>()?;
    let payload = parse_payload(
        &parse_tail(&mut cursor, bytes)?,
        &operations,
        &report,
        pins.sponsorless(),
    )?;
    let evidence = derive(&payload, pins.schedule())?;
    compare_payload(&evidence, &payload, pins)?;
    let recorded_refusal_detail = operations
        .last()
        .ok_or(Refusal::CorpusCensus)?
        .detail
        .clone();
    Ok(ValidatedMaturityCorpus {
        evidence,
        report,
        fee_floors: pins.fee_floors(),
        capture_suite,
        capabilities,
        exchanges: payload.exchanges,
        recorded_refusal_detail,
    })
}

fn validate_inputs(
    files: &[MaturityArchiveFile<'_>],
    pins: &MaturityCorpusPins<'_>,
) -> ImportResult<ValidatedMaturityCorpus> {
    validate_census(files, pins)?;
    validate_manifest(files, pins.manifest_sha256())?;
    let report = parse_report(files[3].bytes, pins)?;
    binding(
        "manifest-sha256",
        report.manifest_digest == pins.manifest_sha256(),
    )?;
    binding(
        "pinned-ceremony-digest",
        report.ceremony_digest == pins.ceremony_sha256(),
    )?;
    // The earlier admitted run exposes these deployment facts through readers.
    // Comparing them avoids a second authored network or genesis identity.
    let operator = crate::live_corpus_native_operator::run_of_record().map_err(|_| {
        Refusal::CrossFileBinding {
            field: "operator-deployment",
        }
    })?;
    binding(
        "development-network-id",
        report.network_id == operator.report().network_id(),
    )?;
    binding(
        "development-genesis-id",
        report.genesis_id == operator.report().genesis_id(),
    )?;
    let mut timing = Cursor::new("maturity timing", files[1].bytes)?;
    timing.exact("timing-schema 1")?;
    timing.exact("ceremony-id report")?;
    timing.exact("status passed")?;
    timing.done()?;
    admit_capture(files[0].bytes, report, pins)
}

/// Admits four pinned maturity files through byte bindings, closed grammars and exact planner replay.
///
/// The payload supplies no standing by authority. A refusal and an acceptance use the same replay discipline with different pinned schedules and sponsorless grammars.
///
/// # Errors
/// Returns the refusal naming the first failed admission layer, preserving planner refusals whole.
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton or the public linker loses a retained constructor, which neither published input can arrange.
#[must_use = "maturity corpus admission can refuse a file or replay binding"]
pub fn admit_maturity_corpus<'a>(
    files: &[MaturityArchiveFile<'a>; 4],
    pins: &MaturityCorpusPins<'a>,
) -> Result<ValidatedMaturityCorpus, MaturityCorpusImportRefusal> {
    validate_inputs(files, pins)
}

/// Admits the immutable historical maturity archive through its own pins.
///
/// The payload is checked against the derived evidence; it supplies no standing by authority. The admitted relay refusal answers the declared boundary and leaves acceptance outstanding through both routes.
///
/// # Errors
/// Returns the refusal naming the first failed admission layer, preserving planner refusals whole.
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton or the public linker loses a retained constructor, which neither published input can arrange.
pub fn maturity_run_of_record()
-> Result<&'static ValidatedMaturityCorpus, MaturityCorpusImportRefusal> {
    static CORPUS: OnceLock<ImportResult<ValidatedMaturityCorpus>> = OnceLock::new();
    match CORPUS.get_or_init(|| admit_maturity_corpus(&FILES, &HISTORICAL_PINS)) {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}
/// Replays the once-admitted immutable archive under an explicit witness selection.
///
/// Every reconstructed request is compared with its recorded counterpart before
/// the responses establish evidence. The archive's byte bindings and admission
/// remain those of [`maturity_run_of_record`].
///
/// # Errors
/// Preserves archive admission refusals and the planner's schedule or exact-replay
/// refusals through [`MaturityCorpusImportRefusal::Derivation`].
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton or the public linker
/// loses a retained constructor, which neither published input can arrange.
#[must_use = "archive replay can refuse the selected schedule or recorded exchanges"]
pub fn replay_maturity_run_of_record(
    selection: MaturityWitnessSelection,
) -> Result<MaturityNativeEvidence, MaturityCorpusImportRefusal> {
    let corpus = maturity_run_of_record()?;
    MaturityNativeEvidence::from_transcript(
        corpus.evidence().identity().clone(),
        corpus.evidence().branch(),
        selection,
        corpus.exchanges(),
    )
    .map_err(Refusal::Derivation)
}

/// Admits the immutable accepted variable-metadata archive through its own pins.
///
/// The executor's development node ran with `-minrelaytxfee=0` and `-blockmintxfee=0`. Admission establishes acceptance in that environment; it does not establish relayability under a positive fee floor, authenticate root freshness, or promote an artifact.
///
/// # Errors
/// Returns the first failed byte, grammar or planner replay binding.
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton or the public linker loses a retained constructor, which neither published input can arrange.
#[must_use = "accepted corpus admission can refuse a file or replay binding"]
pub fn maturity_variable_run_of_record()
-> Result<&'static ValidatedMaturityCorpus, MaturityCorpusImportRefusal> {
    static CORPUS: OnceLock<ImportResult<ValidatedMaturityCorpus>> = OnceLock::new();
    match CORPUS.get_or_init(|| admit_maturity_corpus(&VARIABLE_FILES, &VARIABLE_PINS)) {
        Ok(corpus) => Ok(corpus),
        Err(refusal) => Err(refusal.clone()),
    }
}

/// Replays the once-admitted accepted archive under an explicit witness selection.
///
/// # Errors
/// Preserves archive admission refusals and planner schedule or exact-replay refusals.
///
/// # Panics
/// Panics only if the fixed architecture omits its singleton or the public linker loses a retained constructor, which neither published input can arrange.
#[must_use = "accepted archive replay can refuse the selection or exchanges"]
pub fn replay_maturity_variable_run_of_record(
    selection: MaturityWitnessSelection,
) -> Result<MaturityNativeEvidence, MaturityCorpusImportRefusal> {
    let corpus = maturity_variable_run_of_record()?;
    MaturityNativeEvidence::from_transcript(
        corpus.evidence().identity().clone(),
        corpus.evidence().branch(),
        selection,
        corpus.exchanges(),
    )
    .map_err(Refusal::Derivation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maturity_native::MaturityAnnouncementPlanner;
    use target_elements_conformance::executor::TargetOperationPlanner;
    use target_elements_conformance::protocol::{
        FundedOutput, NativeResourceObservation, WireOutpoint,
    };
    use transaction::bytes::TargetTransaction;

    #[test]
    fn admitted_archives_disclose_the_executor_fee_arguments() {
        let historical = maturity_run_of_record().expect("historical archive");
        let accepted = maturity_variable_run_of_record().expect("accepted archive");
        for corpus in [historical, accepted] {
            assert_eq!(corpus.fee_floors(), MATURITY_DISCLOSED_FEE_FLOORS);
            assert_eq!(
                corpus.fee_floors().provenance(),
                MaturityPremiseProvenance::ExecutorArguments
            );
        }
        let floors = MATURITY_DISCLOSED_FEE_FLOORS;
        assert_eq!(floors.min_relay_tx_fee(), "0");
        assert_eq!(floors.block_min_tx_fee(), "0");
        let driver_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("scripts/elements-native-executor.py");
        let driver = std::fs::read_to_string(driver_path).expect("native executor source");
        assert!(driver.contains("-minrelaytxfee=0"));
        assert!(driver.contains("-blockmintxfee=0"));
        assert!(driver.contains(&format!("-minrelaytxfee={}", floors.min_relay_tx_fee())));
        assert!(driver.contains(&format!("-blockmintxfee={}", floors.block_min_tx_fee())));
    }

    #[test]
    fn retained_whole_metadata_replays_all_three_archived_requests_exactly() {
        let corpus = maturity_run_of_record().expect("admitted archive");
        assert_eq!(
            MATURITY_RUN_SCHEDULE,
            tapscript::StateWitnessSchedule::WholeMetadata
        );
        assert_eq!(corpus.schedule(), MATURITY_RUN_SCHEDULE);
        assert_eq!(corpus.exchanges().len(), 3);
        let evidence = replay_maturity_run_of_record(MaturityWitnessSelection::Retained(
            MATURITY_RUN_SCHEDULE,
        ))
        .expect("exact replay of each recorded request");
        assert_eq!(&evidence, corpus.evidence());
        assert_eq!(evidence.schedule(), corpus.schedule());
    }

    #[test]
    fn archive_replay_refuses_a_different_leafs_transcript() {
        let schedule = tapscript::StateWitnessSchedule::VariableMetadata;
        assert_eq!(
            replay_maturity_run_of_record(MaturityWitnessSelection::Retained(schedule)),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position: 0 }
            ))
        );
    }

    #[test]
    fn archive_replay_refuses_whole_metadata_emission() {
        let schedule = MATURITY_RUN_SCHEDULE;
        assert_eq!(
            replay_maturity_run_of_record(MaturityWitnessSelection::Emission(schedule)),
            Err(Refusal::Derivation(MaturityNativePlanRefusal::Closure(
                Box::new(
                    crate::maturity_closure::MaturityClosureRefusal::ReplayOnlySchedule {
                        schedule
                    },
                )
            )))
        );
    }

    struct OwnedCorpus {
        bytes: [Vec<u8>; 4],
    }

    impl OwnedCorpus {
        fn new() -> Self {
            Self {
                bytes: FILES.map(|file| file.bytes.to_vec()),
            }
        }

        fn inputs(&self) -> [MaturityArchiveFile<'_>; 4] {
            std::array::from_fn(|index| MaturityArchiveFile {
                name: FILES[index].name,
                bytes: &self.bytes[index],
            })
        }

        fn pinned(&self) -> ImportResult<ValidatedMaturityCorpus> {
            admit_maturity_corpus(&self.inputs(), &HISTORICAL_PINS)
        }

        fn replace(&mut self, index: usize, from: &str, to: &str) {
            let text = std::str::from_utf8(&self.bytes[index]).expect("text fixture");
            assert!(text.contains(from), "missing mutation operand");
            self.bytes[index] = text.replacen(from, to, 1).into_bytes();
        }

        fn rebind(&mut self, content: bool) -> ImportResult<ValidatedMaturityCorpus> {
            self.rebind_for(
                content,
                MATURITY_RUN_SCHEDULE,
                MaturitySponsorlessExpectation::RelayRefusal {
                    detail: "bad-witness-nonstandard",
                },
            )
        }

        fn rebind_for(
            &mut self,
            content: bool,
            schedule: tapscript::StateWitnessSchedule,
            sponsorless: MaturitySponsorlessExpectation,
        ) -> ImportResult<ValidatedMaturityCorpus> {
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
            let pins = MaturityCorpusPins::new(
                schedule,
                (HISTORICAL_PINS.names, sizes),
                (&manifest_hash, &address, &capture_hash),
                sponsorless,
                HISTORICAL_PINS.cargo_path,
                HISTORICAL_PINS.framework_revision,
                HISTORICAL_PINS.funding_count,
                MATURITY_DISCLOSED_FEE_FLOORS,
            );
            admit_maturity_corpus(&self.inputs(), &pins)
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

    fn scripted_response(step: &OperationStep) -> NativeOperationResponse {
        let mut response = NativeOperationResponse {
            schema: target_elements_conformance::protocol::NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            issued_asset: None,
            funded_outputs: Vec::new(),
            confidential_funded_outputs: Vec::new(),
            mined_readback: None,
            accepted_txid: None,
            sponsor_witness: Vec::new(),
            script_path_witness: Vec::new(),
            signer_public_key: None,
            signed_profile: None,
            signing_genesis: None,
            signature_bound_to: None,
            resources: NativeResourceObservation::default(),
        };
        match step.subject() {
            OperationSubject::Funding(subject) => {
                let asset = format!("01{}fe", "55".repeat(30));
                if subject.issue_asset {
                    response.issued_asset = Some(asset.clone());
                }
                response.funded_outputs.push(FundedOutput {
                    outpoint: WireOutpoint {
                        txid: format!("02{}fd", "66".repeat(30)),
                        vout: if subject.issue_asset { 3 } else { 7 },
                    },
                    asset,
                    amount_satoshis: subject.amount_per_output,
                    script: hex_bytes(&subject.output_program),
                });
            }
            OperationSubject::Submission(subject) => {
                let candidate =
                    TargetTransaction::decode(&subject.transaction_bytes).expect("decode");
                let identity = Txid::from_internal(tagged::sha256(&tagged::sha256(
                    &candidate.encode_without_witness(),
                )))
                .to_target_display();
                response.accepted_txid = Some(identity.clone());
                response.mined_readback = Some(MinedFundingReadback {
                    transaction_id: identity,
                    witness_transaction_id: Txid::from_internal(tagged::sha256(&tagged::sha256(
                        &subject.transaction_bytes,
                    )))
                    .to_target_display(),
                    block_hash: "77".repeat(32),
                    block_height: 11,
                    raw_transaction: subject.transaction_bytes.clone(),
                });
            }
            _ => panic!("scripted maturity exchange"),
        }
        response.validate_shape().expect("scripted shape");
        response
    }

    fn scripted_variable_run() -> (
        Vec<(OperationStep, NativeOperationResponse)>,
        MaturityNativeEvidence,
    ) {
        let historical = maturity_run_of_record().expect("historical corpus");
        let identity = historical.evidence().identity().clone();
        let branch = historical.evidence().branch();
        let selection =
            MaturityWitnessSelection::Retained(tapscript::StateWitnessSchedule::VariableMetadata);
        let mut planner = MaturityAnnouncementPlanner::new(identity.clone(), branch, selection)
            .expect("variable planner");
        let mut current = planner.next_step(None).expect("first step");
        while let Some(step) = current {
            let response = scripted_response(&step);
            current = planner
                .next_step(Some((step.case(), &response)))
                .expect("settled step");
        }
        let exchanges = planner.completed_transcript().expect("complete").to_vec();
        let evidence =
            MaturityNativeEvidence::from_transcript(identity, branch, selection, &exchanges)
                .expect("exact replay");
        (exchanges, evidence)
    }

    fn write_scripted_field(out: &mut String, name: &str, bytes: &[u8]) {
        let _ = writeln!(out, "{name} {} {}", bytes.len(), hex_bytes(bytes));
    }

    fn scripted_operation_lines(exchanges: &[(OperationStep, NativeOperationResponse)]) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "operation-count {}", exchanges.len());
        for (index, (step, response)) in exchanges.iter().enumerate() {
            let _ = writeln!(out, "operation {index} begin");
            write_scripted_field(
                &mut out,
                "operation-id",
                format!("operation-{index}").as_bytes(),
            );
            write_scripted_field(
                &mut out,
                "request-id",
                format!("request-{index}").as_bytes(),
            );
            let _ = writeln!(out, "request-role auxiliary");
            let request_bytes = match step.subject() {
                OperationSubject::Funding(_) => &[][..],
                OperationSubject::Submission(subject) => subject.transaction_bytes.as_slice(),
                _ => panic!("maturity subject"),
            };
            write_scripted_field(&mut out, "request-bytes", request_bytes);
            write_scripted_field(
                &mut out,
                "request-subject",
                &serde_json::to_vec(step.subject()).expect("subject JSON"),
            );
            write_scripted_field(
                &mut out,
                "response-id",
                format!("response-{index}").as_bytes(),
            );
            write_scripted_field(
                &mut out,
                "response-request-id",
                format!("request-{index}").as_bytes(),
            );
            write_scripted_field(
                &mut out,
                "response-operation-id",
                format!("operation-{index}").as_bytes(),
            );
            let _ = writeln!(out, "response-verdict accepted");
            let _ = writeln!(out, "response-layer accepted");
            let _ = writeln!(
                out,
                "response-target-identity {}",
                response.accepted_txid.as_deref().unwrap_or("none")
            );
            write_scripted_field(&mut out, "response-detail", b"");
            for line in [
                "attribution-control-request-id none",
                "attribution-control-identity none",
                "mutation-kind none",
                "mutation-locator none",
                "projection-input none",
            ] {
                let _ = writeln!(out, "{line}");
            }
            let _ = writeln!(out, "operation {index} end");
        }
        out
    }

    fn scripted_payload(
        exchanges: &[(OperationStep, NativeOperationResponse)],
        evidence: &MaturityNativeEvidence,
    ) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "maturity-evidence-schema 1");
        write_scripted_field(
            &mut out,
            "deployment-network-id",
            evidence.identity().network_id(),
        );
        write_scripted_field(
            &mut out,
            "deployment-genesis-id",
            evidence.identity().genesis_id(),
        );
        write_scripted_field(
            &mut out,
            "branch-identifier",
            evidence.branch().identifier(),
        );
        let _ = writeln!(out, "branch-checkpoint {}", evidence.branch().checkpoint());
        let _ = writeln!(out, "current-root-freshness unestablished");
        for (step, response) in exchanges {
            let _ = writeln!(out, "maturity-step {}", step.case().step);
            write_scripted_field(
                &mut out,
                "validated-response",
                &serde_json::to_vec(response).expect("response JSON"),
            );
        }
        for observation in evidence.observations() {
            let _ = writeln!(
                out,
                "row {} declared {:?} observed {:?}",
                observation.subject(),
                observation.declared_layer(),
                observation.observed_layer()
            );
        }
        let _ = writeln!(out, "sponsorless-standing {:?}", evidence.standing());
        let MaturityAcceptanceObligation::Established {
            schedule,
            identity,
            readback,
        } = evidence.acceptance_obligation()
        else {
            panic!("variable scripted acceptance")
        };
        let _ = writeln!(out, "sponsorless-acceptance established");
        let _ = writeln!(out, "accepted-schedule {}", schedule.name());
        let _ = writeln!(out, "accepted-identity {}", identity.to_target_display());
        let _ = writeln!(
            out,
            "accepted-witness-identity {}",
            readback.witness_identity().to_target_display()
        );
        let _ = writeln!(
            out,
            "accepted-block {} {}",
            hex_bytes(readback.block_hash()),
            readback.block_height()
        );
        out
    }

    fn accepted_corpus() -> OwnedCorpus {
        let (exchanges, evidence) = scripted_variable_run();
        let operations = scripted_operation_lines(&exchanges);
        let payload = scripted_payload(&exchanges, &evidence);
        let historical = std::str::from_utf8(FILES[0].bytes).expect("historical capture");
        let prefix = historical
            .split_once("operation-count 3\n")
            .expect("operations")
            .0;
        let after_payload = historical
            .split_once("legacy-rendering ")
            .expect("rendering")
            .1;
        let tail = after_payload.split_once('\n').expect("tail").1;
        let mut capture = String::new();
        capture.push_str(prefix);
        capture.push_str(&operations);
        let _ = writeln!(
            capture,
            "legacy-rendering {} {}",
            payload.len(),
            hex_bytes(payload.as_bytes())
        );
        capture.push_str(tail);
        let mut owned = OwnedCorpus::new();
        owned.bytes[0] = capture.into_bytes();
        owned
    }

    fn flip_recorded_submission_request(corpus: &mut OwnedCorpus) {
        let text = std::str::from_utf8(&corpus.bytes[0]).expect("capture");
        let (prefix, after_begin) = text.split_once("operation 2 begin\n").expect("submission");
        let (block, suffix) = after_begin
            .split_once("operation 2 end\n")
            .expect("submission end");
        let mut changed = String::new();
        changed.push_str(prefix);
        changed.push_str("operation 2 begin\n");
        let mut changed_byte = None;
        for line in block.lines() {
            if let Some(value) = line.strip_prefix("request-bytes ") {
                let (_, hex) = value.split_once(' ').expect("request bytes");
                let mut bytes = decode_hex(hex).expect("request encoding");
                bytes[0] ^= 1;
                changed_byte = Some(bytes[0]);
                write_scripted_field(&mut changed, "request-bytes", &bytes);
            } else if let Some(value) = line.strip_prefix("request-subject ") {
                let (_, hex) = value.split_once(' ').expect("subject bytes");
                let mut subject: serde_json::Value =
                    serde_json::from_slice(&decode_hex(hex).expect("subject encoding"))
                        .expect("subject JSON");
                subject["transaction_bytes"][0] =
                    serde_json::Value::from(changed_byte.expect("request byte"));
                write_scripted_field(
                    &mut changed,
                    "request-subject",
                    &serde_json::to_vec(&subject).expect("subject JSON"),
                );
            } else {
                let _ = writeln!(changed, "{line}");
            }
        }
        changed.push_str("operation 2 end\n");
        changed.push_str(suffix);
        corpus.bytes[0] = changed.into_bytes();
    }

    #[test]
    fn historical_pins_and_public_admission_equal_the_record_loader() {
        assert_eq!(HISTORICAL_PINS.schedule(), MATURITY_RUN_SCHEDULE);
        assert_eq!(HISTORICAL_PINS.names(), &FILES.map(|file| file.name));
        assert_eq!(HISTORICAL_PINS.sizes(), &FILE_SIZES);
        assert_eq!(HISTORICAL_PINS.manifest_sha256(), MATURITY_MANIFEST_SHA256);
        assert_eq!(HISTORICAL_PINS.run_address(), MATURITY_RUN_ADDRESS);
        assert_eq!(HISTORICAL_PINS.ceremony_sha256(), CEREMONY_SHA256);
        assert_eq!(
            HISTORICAL_PINS.cargo_path(),
            "/workspace/toolchains/cargo/bin/cargo"
        );
        assert_eq!(
            HISTORICAL_PINS.framework_revision(),
            MaturityFrameworkRevision::Unrecorded
        );
        assert_eq!(HISTORICAL_PINS.funding_count(), 0);
        assert_eq!(
            HISTORICAL_PINS.sponsorless(),
            MaturitySponsorlessExpectation::RelayRefusal {
                detail: "bad-witness-nonstandard"
            }
        );
        assert_eq!(
            admit_maturity_corpus(&FILES, &HISTORICAL_PINS),
            Ok(maturity_run_of_record().expect("historical loader").clone())
        );
    }

    #[test]
    fn variable_archive_admits_established_identity_and_exact_mined_readback() {
        let corpus = maturity_variable_run_of_record().expect("accepted archive");
        assert_eq!(corpus.schedule(), MATURITY_VARIABLE_RUN_SCHEDULE);
        assert_eq!(corpus.exchanges().len(), 3);
        assert_eq!(
            corpus.evidence().standing(),
            MaturityNativeStanding::AnsweredAtDeclaredBoundary
        );
        let MaturityAcceptanceObligation::Established {
            schedule,
            identity,
            readback,
        } = corpus.evidence().acceptance_obligation()
        else {
            panic!("accepted archive obligation")
        };
        assert_eq!(*schedule, MATURITY_VARIABLE_RUN_SCHEDULE);
        assert_eq!(*identity, readback.identity());
        assert!(readback.block_height() >= 1);
        let OperationSubject::Submission(subject) = corpus.exchanges()[2].0.subject() else {
            panic!("submission subject")
        };
        assert_eq!(readback.bytes(), subject.transaction_bytes.as_slice());
        let capture = std::str::from_utf8(VARIABLE_FILES[0].bytes()).expect("capture text");
        let response_identity = capture
            .lines()
            .rfind(|line| line.starts_with("response-target-identity "))
            .expect("submission response identity")
            .strip_prefix("response-target-identity ")
            .expect("response field");
        assert_eq!(identity.to_target_display(), response_identity);
        let rendered_hex = capture
            .lines()
            .find_map(|line| line.strip_prefix("legacy-rendering "))
            .expect("payload")
            .split_whitespace()
            .nth(1)
            .expect("payload hex");
        let rendered = decode_hex(rendered_hex).expect("payload bytes");
        let rendered = std::str::from_utf8(&rendered).expect("payload text");
        assert!(rendered.contains(&format!("accepted-identity {response_identity}\n")));
        assert_eq!(
            replay_maturity_variable_run_of_record(MaturityWitnessSelection::Retained(
                MATURITY_VARIABLE_RUN_SCHEDULE
            )),
            Ok(corpus.evidence().clone())
        );
    }

    #[test]
    fn variable_pins_equal_the_embedded_file_facts() {
        assert_eq!(
            VARIABLE_PINS.names(),
            &VARIABLE_FILES.map(|file| file.name())
        );
        assert_eq!(
            VARIABLE_PINS.sizes(),
            &VARIABLE_FILES.map(|file| file.bytes().len())
        );
        assert_eq!(
            VARIABLE_PINS.ceremony_sha256(),
            hex_bytes(&tagged::sha256(VARIABLE_FILES[0].bytes()))
        );
        assert_eq!(
            VARIABLE_PINS.manifest_sha256(),
            hex_bytes(&tagged::sha256(VARIABLE_FILES[2].bytes()))
        );
        assert_eq!(
            VARIABLE_PINS.run_address(),
            hex_bytes(&tagged::sha256(VARIABLE_FILES[3].bytes()))
        );
        assert_eq!(
            VARIABLE_PINS.sponsorless(),
            MaturitySponsorlessExpectation::Acceptance
        );
        let report = std::str::from_utf8(VARIABLE_FILES[3].bytes()).expect("report text");
        let argv_hex = report
            .lines()
            .find_map(|line| line.strip_prefix("cargo-argv "))
            .expect("cargo argv")
            .split_whitespace()
            .nth(1)
            .expect("cargo hex");
        let argv = decode_hex(argv_hex).expect("cargo bytes");
        assert_eq!(
            std::str::from_utf8(&argv).expect("cargo text"),
            format!(
                "{} test -p tripod-vectors --test maturity_native -- --ignored --test-threads=1",
                VARIABLE_PINS.cargo_path()
            )
        );
        let capture = std::str::from_utf8(VARIABLE_FILES[0].bytes()).expect("capture text");
        let framework_hex = capture
            .lines()
            .find_map(|line| line.strip_prefix("handshake-framework-revision 40 "))
            .expect("recorded framework tip");
        let framework = decode_hex(framework_hex).expect("framework bytes");
        assert_eq!(
            std::str::from_utf8(&framework).expect("framework text"),
            maturity_variable_run_of_record()
                .expect("accepted archive")
                .report()
                .intended_tip()
        );
        assert_eq!(
            VARIABLE_PINS.framework_revision(),
            MaturityFrameworkRevision::IntendedTip
        );
        let funding_count = capture
            .lines()
            .find_map(|line| line.strip_prefix("environment-funding-count "))
            .expect("funding count")
            .parse::<usize>()
            .expect("funding number");
        assert_eq!(VARIABLE_PINS.funding_count(), funding_count);
    }

    #[test]
    fn variable_files_under_historical_pins_refuse_corpus_census() {
        assert_eq!(
            admit_maturity_corpus(&VARIABLE_FILES, &HISTORICAL_PINS),
            Err(Refusal::CorpusCensus)
        );
    }

    #[test]
    fn historical_files_under_variable_pins_refuse_corpus_census() {
        assert_eq!(
            admit_maturity_corpus(&FILES, &VARIABLE_PINS),
            Err(Refusal::CorpusCensus)
        );
    }

    #[test]
    fn variable_replay_under_whole_schedule_refuses_transcript_step_mismatch_zero() {
        assert_eq!(
            replay_maturity_variable_run_of_record(MaturityWitnessSelection::Retained(
                tapscript::StateWitnessSchedule::WholeMetadata
            )),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position: 0 }
            ))
        );
    }

    #[test]
    fn wrong_cargo_path_pin_refuses_run_report_grammar() {
        let pins = MaturityCorpusPins {
            cargo_path: "/another/cargo",
            ..VARIABLE_PINS
        };
        assert_eq!(
            admit_maturity_corpus(&VARIABLE_FILES, &pins),
            Err(Refusal::RunReportGrammar)
        );
    }

    #[test]
    fn unrecorded_framework_pin_refuses_cross_file_binding() {
        let pins = MaturityCorpusPins {
            framework_revision: MaturityFrameworkRevision::Unrecorded,
            ..VARIABLE_PINS
        };
        assert_eq!(
            admit_maturity_corpus(&VARIABLE_FILES, &pins),
            Err(Refusal::CrossFileBinding {
                field: "handshake-framework-revision"
            })
        );
    }

    #[test]
    fn wrong_funding_count_pin_refuses_transcript_grammar() {
        let pins = MaturityCorpusPins {
            funding_count: VARIABLE_PINS.funding_count() + 1,
            ..VARIABLE_PINS
        };
        assert!(matches!(
            admit_maturity_corpus(&VARIABLE_FILES, &pins),
            Err(Refusal::TranscriptGrammar { .. })
        ));
    }

    #[test]
    fn scripted_variable_corpus_admits_established_identity_and_readback() {
        let mut files = accepted_corpus();
        let admitted = files
            .rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance,
            )
            .expect("accepted corpus");
        let (_, scripted) = scripted_variable_run();
        assert_eq!(admitted.evidence(), &scripted);
        assert_eq!(
            admitted.schedule(),
            tapscript::StateWitnessSchedule::VariableMetadata
        );
        assert_eq!(admitted.recorded_refusal_detail(), "");
        let MaturityAcceptanceObligation::Established {
            identity, readback, ..
        } = admitted.evidence().acceptance_obligation()
        else {
            panic!("accepted corpus obligation")
        };
        assert_eq!(*identity, readback.identity());
        let OperationSubject::Submission(subject) = admitted.exchanges()[2].0.subject() else {
            panic!("submission bytes")
        };
        assert_eq!(readback.bytes(), subject.transaction_bytes.as_slice());
    }

    #[test]
    fn accepted_files_with_whole_schedule_refuse_transcript_step_mismatch_zero() {
        let mut files = accepted_corpus();
        assert_eq!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::WholeMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position: 0 }
            ))
        );
    }

    #[test]
    fn historical_files_under_acceptance_refuse_transcript_grammar() {
        let mut files = OwnedCorpus::new();
        assert!(matches!(
            files.rebind_for(
                true,
                MATURITY_RUN_SCHEDULE,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::TranscriptGrammar { .. })
        ));
    }

    #[test]
    fn accepted_files_under_relay_refusal_pins_refuse_transcript_grammar() {
        let mut files = accepted_corpus();
        assert!(matches!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::RelayRefusal {
                    detail: "bad-witness-nonstandard"
                }
            ),
            Err(Refusal::TranscriptGrammar { .. })
        ));
    }

    #[test]
    fn wrong_capture_and_payload_identity_refuse_derivation_submission_readback_mismatch() {
        let mut files = accepted_corpus();
        let (_, evidence) = scripted_variable_run();
        let MaturityAcceptanceObligation::Established { identity, .. } =
            evidence.acceptance_obligation()
        else {
            panic!("identity")
        };
        let original = identity.to_target_display();
        let wrong = "99".repeat(32);
        files.replace(
            0,
            &format!("response-target-identity {original}"),
            &format!("response-target-identity {wrong}"),
        );
        files.payload_replace(
            &format!("accepted-identity {original}"),
            &format!("accepted-identity {wrong}"),
        );
        assert_eq!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::SubmissionReadbackMismatch
            ))
        );
    }

    #[test]
    fn wrong_payload_witness_identity_refuses_derivation_submission_readback_mismatch() {
        let mut files = accepted_corpus();
        let (_, evidence) = scripted_variable_run();
        let MaturityAcceptanceObligation::Established { readback, .. } =
            evidence.acceptance_obligation()
        else {
            panic!("readback")
        };
        let original = readback.witness_identity().to_target_display();
        files.payload_replace(
            &format!("accepted-witness-identity {original}"),
            &format!("accepted-witness-identity {}", "99".repeat(32)),
        );
        assert_eq!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::SubmissionReadbackMismatch
            ))
        );
    }

    #[test]
    fn payload_identity_different_from_capture_refuses_cross_file_binding() {
        let mut files = accepted_corpus();
        let (_, evidence) = scripted_variable_run();
        let MaturityAcceptanceObligation::Established { identity, .. } =
            evidence.acceptance_obligation()
        else {
            panic!("identity")
        };
        files.payload_replace(
            &format!("accepted-identity {}", identity.to_target_display()),
            &format!("accepted-identity {}", "99".repeat(32)),
        );
        assert_eq!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::CrossFileBinding {
                field: "accepted-identity"
            })
        );
    }

    #[test]
    fn flipped_recorded_request_refuses_derivation_transcript_step_mismatch_two() {
        let mut files = accepted_corpus();
        flip_recorded_submission_request(&mut files);
        assert_eq!(
            files.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position: 2 }
            ))
        );
    }

    #[test]
    fn incomplete_and_refused_identity_or_established_payload_refuse_transcript_grammar() {
        let mut incomplete = accepted_corpus();
        incomplete.replace(0, "operation-count 3", "operation-count 2");
        assert!(matches!(
            incomplete.rebind_for(
                true,
                tapscript::StateWitnessSchedule::VariableMetadata,
                MaturitySponsorlessExpectation::Acceptance
            ),
            Err(Refusal::TranscriptGrammar { .. })
        ));
        let mut refused_identity = OwnedCorpus::new();
        refused_identity.replace(
            0,
            "response-layer relay-policy-rejection\nresponse-target-identity none",
            &format!(
                "response-layer relay-policy-rejection\nresponse-target-identity {}",
                "99".repeat(32)
            ),
        );
        assert!(matches!(
            refused_identity.rebind(true),
            Err(Refusal::TranscriptGrammar { .. })
        ));
        let mut refused_established = OwnedCorpus::new();
        refused_established.payload_replace(
            "sponsorless-acceptance outstanding",
            "sponsorless-acceptance established",
        );
        assert!(matches!(
            refused_established.rebind(true),
            Err(Refusal::TranscriptGrammar { .. })
        ));
    }

    #[test]
    fn the_admitted_bytes_derive_the_boundary_and_leave_acceptance_outstanding() {
        let corpus = maturity_run_of_record().expect("admitted corpus");
        let report = corpus.report();
        assert_eq!(corpus.content_address(), MATURITY_RUN_ADDRESS);
        assert_eq!(report.run_address(), MATURITY_RUN_ADDRESS);
        assert_eq!(report.manifest_digest(), MATURITY_MANIFEST_SHA256);
        assert_eq!(report.ceremony_digest(), CEREMONY_SHA256);
        assert_eq!(report.suite_commit(), corpus.capture_suite_commit());
        assert_eq!(report.suite_tree(), corpus.capture_suite_tree());
        assert_eq!(report.expected_tip(), report.intended_tip());
        assert!(report.expected_tip().starts_with(report.binary_revision()));
        assert_eq!(report.adapter_name(), "elements-native-executor");
        assert_eq!(report.adapter_version(), "2.1.0");
        assert_eq!(report.node_name(), "Elements");
        assert_eq!(
            report.node_version(),
            format!(
                "Elements Core daemon version v28.99.0-{}",
                report.binary_revision()
            )
        );
        assert_eq!(report.protocol_revision(), 8);
        assert_eq!(report.environment(), "development");
        assert_eq!(report.target_contract(), "elements-tapscript-v2");
        let operator =
            crate::live_corpus_native_operator::run_of_record().expect("operator corpus");
        assert_eq!(report.network_id(), operator.report().network_id());
        assert_eq!(report.genesis_id(), operator.report().genesis_id());
        assert_eq!(corpus.capabilities(), CAPABILITIES);
        assert_eq!(corpus.recorded_refusal_detail(), "bad-witness-nonstandard");
        let evidence = corpus.evidence();
        assert_eq!(
            evidence.identity().network_id(),
            &digest(report.network_id()).expect("network")
        );
        assert_eq!(
            evidence.identity().genesis_id(),
            &digest(report.genesis_id()).expect("genesis")
        );
        assert_eq!(evidence.branch().identifier(), &[0x41; 32]);
        assert_eq!(evidence.branch().checkpoint(), 7);
        assert_eq!(
            evidence.standing(),
            MaturityNativeStanding::AnsweredAtDeclaredBoundary
        );
        assert_eq!(
            evidence.acceptance_obligation(),
            &MaturityAcceptanceObligation::Outstanding {
                routes: [
                    MaturityAcceptanceRoute::RelayWitnessRestructure,
                    MaturityAcceptanceRoute::BlockLayerSubmissionSubject
                ],
            }
        );
        assert_eq!(corpus.exchanges().len(), 3);
        for ((step, response), observation) in
            corpus.exchanges().iter().zip(evidence.observations())
        {
            assert_eq!(&response.case, step.case());
            assert_eq!(step.case().step, observation.subject());
            assert_eq!(response.observed_layer, observation.observed_layer());
        }
    }

    #[test]
    fn every_file_byte_flip_stops_at_its_outer_layer() {
        for index in 0..4 {
            let mut corpus = OwnedCorpus::new();
            corpus.bytes[index][0] ^= 1;
            let expected = match index {
                0 | 1 => Refusal::ManifestDigest {
                    name: FILES[index].name.to_owned(),
                },
                2 => Refusal::ManifestHash,
                3 => Refusal::RunReportGrammar,
                _ => unreachable!(),
            };
            assert_eq!(corpus.pinned(), Err(expected));
        }
    }

    #[test]
    fn census_and_every_file_length_are_fixed() {
        assert_eq!(
            validate_census(&FILES[..3], &HISTORICAL_PINS),
            Err(Refusal::CorpusCensus)
        );
        let mut files = FILES;
        files.swap(0, 1);
        assert_eq!(
            validate_census(&files, &HISTORICAL_PINS),
            Err(Refusal::CorpusCensus)
        );
        for (index, expected) in FILE_SIZES.iter().enumerate() {
            let mut corpus = OwnedCorpus::new();
            corpus.bytes[index].pop();
            assert_eq!(
                corpus.pinned(),
                Err(Refusal::FileSize {
                    name: FILES[index].name.to_owned(),
                    expected: *expected,
                    actual: expected - 1,
                })
            );
        }
    }

    #[test]
    fn manifest_lines_are_closed_sorted_and_digest_checked() {
        let manifest = std::str::from_utf8(FILES[2].bytes).expect("manifest");
        let lines: Vec<_> = manifest.lines().collect();
        for malformed in [
            format!("{manifest}extra\n"),
            format!("{}\n{}\n", lines[1], lines[0]),
            format!("{}\n", lines[0]),
            manifest.replace("  ", " "),
            manifest.replace(".timing", ".other"),
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
        let mut corpus = OwnedCorpus::new();
        corpus.bytes[2][0] = b'0';
        assert_eq!(
            validate_manifest(
                &corpus.inputs(),
                &hex_bytes(&tagged::sha256(&corpus.bytes[2]))
            ),
            Err(Refusal::ManifestDigest {
                name: FILES[0].name.to_owned()
            })
        );
    }

    #[test]
    fn report_grammar_closes_counts_eligibility_order_and_provenance() {
        for (from, to) in [
            ("suite-clean yes", "suite-clean no"),
            ("observed-test-count 1", "observed-test-count 2"),
            ("observed-ceremony-count 1", "observed-ceremony-count 0"),
            ("observed-setup-count 0", "observed-setup-count 1"),
            ("diagnostics-present yes", "diagnostics-present no"),
            ("test-failed-count 0", "test-failed-count 1"),
            ("test-ignored-count 0", "test-ignored-count 1"),
            ("cargo-exit-code 0", "cargo-exit-code 1"),
            ("protocol-revision 8", "protocol-revision 7"),
            ("eligible yes", "eligible no"),
            (
                "expected-test-count 1\nobserved-test-count 1",
                "observed-test-count 1\nexpected-test-count 1",
            ),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.replace(3, from, to);
            assert_eq!(
                parse_report(&corpus.bytes[3], &HISTORICAL_PINS),
                Err(Refusal::RunReportGrammar)
            );
        }
        let mut corpus = OwnedCorpus::new();
        corpus.bytes[3].extend_from_slice(b"extra field\n");
        assert_eq!(
            parse_report(&corpus.bytes[3], &HISTORICAL_PINS),
            Err(Refusal::RunReportGrammar)
        );
    }

    #[test]
    fn a_grammatical_report_still_needs_its_pinned_address() {
        let mut corpus = OwnedCorpus::new();
        let text = std::str::from_utf8(&corpus.bytes[3]).expect("report");
        let start = text.find("suite-tree ").expect("tree") + "suite-tree ".len();
        corpus.bytes[3][start] = b'2';
        assert_eq!(corpus.pinned(), Err(Refusal::RunReportAddress));
    }

    #[test]
    fn outer_rebinding_does_not_replace_the_capture_content_hash() {
        let mut corpus = OwnedCorpus::new();
        corpus.payload_replace(
            "sponsorless-acceptance outstanding",
            "sponsorless-acceptance accepted",
        );
        assert_eq!(corpus.rebind(false), Err(Refusal::CaptureContentHash));
    }

    #[test]
    fn report_and_capture_bind_each_source_identity() {
        for field in ["suite-commit", "suite-tree"] {
            let mut corpus = OwnedCorpus::new();
            let text = std::str::from_utf8(&corpus.bytes[0]).expect("capture");
            let start = text.find(&format!("{field} ")).expect("field") + field.len() + 1;
            corpus.bytes[0][start] = b'0';
            assert_eq!(
                corpus.rebind(true),
                Err(Refusal::CrossFileBinding { field })
            );
        }
    }

    #[test]
    fn handshake_and_environment_bind_the_report() {
        for (from, to, field) in [
            (
                "handshake-protocol-schema 8",
                "handshake-protocol-schema 7",
                "protocol-revision",
            ),
            (
                "deployment-environment development",
                "deployment-environment other",
                "deployment-environment",
            ),
            (
                "handshake-framework-revision 0 ",
                "handshake-framework-revision 1 61",
                "handshake-framework-revision",
            ),
            (
                "response-request-id 9 726571756573742d30",
                "response-request-id 9 726571756573742d31",
                "response-request-id",
            ),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.replace(0, from, to);
            assert_eq!(
                corpus.rebind(true),
                Err(Refusal::CrossFileBinding { field })
            );
        }
    }

    #[test]
    fn the_capture_closes_schema_rosters_roles_mutations_and_terminal_state() {
        for (from, to) in [
            ("native-capture-schema 2", "native-capture-schema 1"),
            (
                "environment-capability-count 12",
                "environment-capability-count 11",
            ),
            (
                "environment-capability 0 failure-class-reporting",
                "environment-capability 0 unknown",
            ),
            ("environment-funding-count 0", "environment-funding-count 1"),
            ("operation-count 3", "operation-count 2"),
            ("request-role auxiliary", "request-role control"),
            ("response-verdict refused", "response-verdict accepted"),
            (
                "response-layer relay-policy-rejection",
                "response-layer script-path-rejection",
            ),
            ("mutation-kind none", "mutation-kind witness"),
            ("mutation-locator none", "mutation-locator 0"),
            ("terminal-state complete", "terminal-state incomplete"),
            ("native-capture-end report", "native-capture-end other"),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.replace(0, from, to);
            assert!(
                matches!(corpus.rebind(true), Err(Refusal::TranscriptGrammar { .. })),
                "{from}"
            );
        }
    }

    #[test]
    fn byte_fields_and_line_endings_have_one_spelling() {
        for bytes in [
            b"field 01 00\n".as_slice(),
            b"field 1 FF\n",
            b"field 2 00\n",
            b"field 1 0g\n",
        ] {
            let mut cursor = Cursor::new("test", bytes).expect("lines");
            assert!(matches!(
                cursor.bytes("field"),
                Err(Refusal::TranscriptGrammar { .. })
            ));
        }
        for bytes in [
            b"field 0 \r\n".as_slice(),
            b"field 0 ",
            b"field 0 \n\n",
            b"\xff\n",
        ] {
            assert!(matches!(
                Cursor::new("test", bytes),
                Err(Refusal::TranscriptGrammar { .. })
            ));
        }
    }

    #[test]
    fn timing_is_a_closed_passed_record() {
        let mut corpus = OwnedCorpus::new();
        corpus.replace(1, "status passed", "status failed");
        assert!(
            matches!(corpus.rebind(true), Err(Refusal::TranscriptGrammar { name, .. }) if name == "maturity timing")
        );
    }

    #[test]
    fn recorded_request_subjects_match_the_protocol_serializer_exactly() {
        let corpus = maturity_run_of_record().expect("corpus");
        let text = std::str::from_utf8(FILES[0].bytes).expect("capture");
        for (line, (step, _)) in text
            .lines()
            .filter(|line| line.starts_with("request-subject "))
            .zip(corpus.exchanges())
        {
            let bytes = decode_hex(line.split(' ').nth(2).expect("hex")).expect("bytes");
            assert_eq!(
                bytes,
                serde_json::to_vec(step.subject()).expect("serialize")
            );
        }
        for malformed in [
            "{\"transaction_bytes\":[256]}",
            "{\"transaction_bytes\":[01]}",
            "{\"transaction_bytes\":[1],\"extra\":0}",
            "{\"transaction_bytes\":[1],\"transaction_bytes\":[2]}",
        ] {
            assert_eq!(parse_subject(malformed, 2), None);
        }
    }

    #[test]
    fn payload_context_and_step_census_are_closed() {
        for (from, to) in [
            ("branch-identifier 32 41", "branch-identifier 32 42"),
            ("branch-checkpoint 7", "branch-checkpoint 8"),
            (
                "current-root-freshness unestablished",
                "current-root-freshness established",
            ),
            (
                "maturity-step issue-maturity-singleton",
                "maturity-step fund-maturity-predecessor",
            ),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.payload_replace(from, to);
            assert!(matches!(
                corpus.rebind(true),
                Err(Refusal::TranscriptGrammar { .. })
            ));
        }
        let mut corpus = OwnedCorpus::new();
        let network = maturity_run_of_record()
            .expect("corpus")
            .report()
            .network_id();
        corpus.payload_replace(
            &format!("deployment-network-id 32 {network}"),
            &format!("deployment-network-id 32 {}", "00".repeat(32)),
        );
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::CrossFileBinding {
                field: "payload-network-id"
            })
        );
    }

    #[test]
    fn every_payload_row_and_claim_is_compared_with_replay() {
        for (from, to, field) in [
            (
                "row issue-maturity-singleton declared None",
                "row issue-maturity-singleton declared Some(Accepted)",
                "issue-maturity-singleton",
            ),
            (
                "row fund-maturity-predecessor declared None",
                "row fund-maturity-predecessor declared Some(Accepted)",
                "fund-maturity-predecessor",
            ),
            (
                "row sponsorless declared Some(RelayPolicyRejection)",
                "row sponsorless declared Some(Accepted)",
                "sponsorless",
            ),
            (
                "sponsorless-standing AnsweredAtDeclaredBoundary",
                "sponsorless-standing ObservedElsewhere",
                "sponsorless-standing",
            ),
            (
                "sponsorless-acceptance outstanding",
                "sponsorless-acceptance accepted",
                "sponsorless-acceptance",
            ),
            (
                "acceptance-route RelayWitnessRestructure",
                "acceptance-route BlockLayerSubmissionSubject",
                "acceptance-routes",
            ),
            (
                "acceptance-route BlockLayerSubmissionSubject",
                "acceptance-route RelaxedPolicy",
                "acceptance-routes",
            ),
        ] {
            let mut corpus = OwnedCorpus::new();
            corpus.payload_replace(from, to);
            assert_eq!(
                corpus.rebind(true),
                Err(Refusal::PayloadDisagreement { field })
            );
        }
    }

    fn payload() -> Payload {
        let corpus = maturity_run_of_record().expect("corpus");
        Payload {
            identity: corpus.evidence().identity().clone(),
            branch: corpus.evidence().branch(),
            exchanges: corpus.exchanges().to_vec(),
            rows: Vec::new(),
            standing: String::new(),
            acceptance: PayloadAcceptance::Outstanding {
                claim: String::new(),
                routes: [String::new(), String::new()],
            },
        }
    }

    #[test]
    fn replay_preserves_each_exact_request_mismatch_and_incomplete_prefix() {
        for position in 0..3 {
            let mut payload = payload();
            let (step, _) = &mut payload.exchanges[position];
            let mut subject = step.subject().clone();
            match &mut subject {
                OperationSubject::Funding(funding) => funding.amount_per_output += 1,
                OperationSubject::Submission(submission) => submission.transaction_bytes[0] ^= 1,
                _ => panic!("only recorded shapes"),
            }
            *step = OperationStep::new(&step.case().step, subject);
            assert_eq!(
                derive(&payload, MATURITY_RUN_SCHEDULE),
                Err(Refusal::Derivation(
                    MaturityNativePlanRefusal::TranscriptStepMismatch { position }
                ))
            );
            let mut payload = self::payload();
            payload.exchanges.truncate(position);
            assert_eq!(
                derive(&payload, MATURITY_RUN_SCHEDULE),
                Err(Refusal::Derivation(
                    MaturityNativePlanRefusal::IncompleteTranscript
                ))
            );
        }
    }

    #[test]
    fn replay_preserves_response_refusals_whole() {
        let mut payload = payload();
        payload.exchanges[0].1.funded_outputs[0].amount_satoshis += 1;
        assert_eq!(
            derive(&payload, MATURITY_RUN_SCHEDULE),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::FundingMismatch
            ))
        );
        let mut payload = self::payload();
        payload.exchanges[0].1.schema = 7;
        assert_eq!(
            derive(&payload, MATURITY_RUN_SCHEDULE),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::ResponseSchema { offered: 7 }
            ))
        );
    }

    #[test]
    fn admission_passes_recorded_request_changes_to_exact_replay() {
        let mut corpus = OwnedCorpus::new();
        let text = std::str::from_utf8(&corpus.bytes[0]).expect("capture");
        let line = text
            .lines()
            .find(|line| line.starts_with("request-subject "))
            .expect("subject")
            .to_owned();
        let decoded = decode_hex(line.split(' ').nth(2).expect("hex")).expect("bytes");
        let subject = String::from_utf8(decoded)
            .expect("JSON")
            .replace("\"amount_per_output\":1", "\"amount_per_output\":2");
        corpus.replace(
            0,
            &line,
            &format!(
                "request-subject {} {}",
                subject.len(),
                hex_bytes(subject.as_bytes())
            ),
        );
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::TranscriptStepMismatch { position: 0 }
            ))
        );
    }

    #[test]
    fn admission_preserves_the_protocol_revision_refusal_from_replay() {
        let mut corpus = OwnedCorpus::new();
        corpus.payload_replace("7b22736368656d61223a38", "7b22736368656d61223a37");
        assert_eq!(
            corpus.rebind(true),
            Err(Refusal::Derivation(
                MaturityNativePlanRefusal::ResponseSchema { offered: 7 }
            ))
        );
    }
}
