//! Immutable compact archive admission through the requirement-keyed schedule.

use crate::compact_campaign::{
    ByteRecipe, CampaignRow, ControlFamily, UnresolvedRecipe, requirement_name,
};
use crate::historical_candidate::{CandidateStanding, CurrentCandidate};
use crate::matrix::EvidenceBoundary;
use compiler::operation_plan::CoverageRequirementId;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;
use std::sync::OnceLock;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::executor::TargetOperationPlanner;
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationSubject,
};
use transaction::{
    InputWitness, Outpoint, TargetInput, TargetOutput, TargetTransaction, ValueField,
};

#[allow(
    clippy::redundant_pub_crate,
    reason = "the shared file also compiles at the integration target root"
)]
pub(crate) mod schedule {
    use crate as candidate;
    include!("compact_campaign_planner.rs");
}

/// Content pin for the exact two-entry manifest.
pub const COMPACT_MANIFEST_SHA256: &str =
    "de6b79d2ddb89c69178273f9f99a19597ffea87c9414ec46589c8b4cc17df75b";
/// Content pin for the eligible report.
pub const COMPACT_REPORT_SHA256: &str =
    "d62e775d68774d9d79e82632a82d0bd889f463ea9b050b2c911706f7843d8de5";
/// Address of the selected fixed schedule and attempt.
pub const COMPACT_RUN_ADDRESS: &str =
    "4cbb74eb49e745d262f5e5087ea544e3c63317988efc616e280bb4d350f7dee0";
const CAPTURE: &str = include_str!(
    "../fixtures/compact-run-of-record/4cbb74eb49e745d262f5e5087ea544e3c63317988efc616e280bb4d350f7dee0.report.capture"
);
const TIMING: &str = include_str!(
    "../fixtures/compact-run-of-record/4cbb74eb49e745d262f5e5087ea544e3c63317988efc616e280bb4d350f7dee0.report.capture.timing"
);
const MANIFEST: &str = include_str!("../fixtures/compact-run-of-record/MANIFEST.sha256");
const REPORT: &str = include_str!("../fixtures/compact-run-of-record/RUN-REPORT");

/// Admission stops at the first failed independent binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompactImportRefusal {
    Manifest,
    Report,
    Grammar,
    Identity,
    Recipe,
    Schedule,
    Settlement,
    Census,
}
type Result<T> = std::result::Result<T, CompactImportRefusal>;

/// A formal argument is separate from every observed refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormalClosure {
    NoSponsorAuthorizationCoordinate,
}

/// The current standing of one native requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeDisposition {
    Observed {
        family: ControlFamily,
        expected: EvidenceBoundary,
        observed: ObservedOutcomeLayer,
    },
    FormallyClosed(FormalClosure),
    Unresolved(UnresolvedRecipe),
    Suspended(ControlFamily),
}

/// Only complete archive replay constructs this value.
#[derive(Debug)]
pub struct CompactCorpus {
    standing: CandidateStanding,
    rows: BTreeMap<CoverageRequirementId, NativeDisposition>,
    controls: BTreeSet<ControlFamily>,
}
impl CompactCorpus {
    #[must_use]
    pub const fn standing(&self) -> &CandidateStanding {
        &self.standing
    }
    #[must_use]
    pub const fn rows(&self) -> &BTreeMap<CoverageRequirementId, NativeDisposition> {
        &self.rows
    }
    #[must_use]
    pub const fn controls(&self) -> &BTreeSet<ControlFamily> {
        &self.controls
    }
}

const fn require(yes: bool, refusal: CompactImportRefusal) -> Result<()> {
    if yes { Ok(()) } else { Err(refusal) }
}
fn hash(bytes: &[u8]) -> String {
    schedule::hex_bytes(&sha256(bytes))
}
fn decode(value: &str) -> Result<Vec<u8>> {
    let (length, text) = value.split_once(' ').ok_or(CompactImportRefusal::Grammar)?;
    let length: usize = length.parse().map_err(|_| CompactImportRefusal::Grammar)?;
    require(
        length.to_string()
            == value
                .split_once(' ')
                .ok_or(CompactImportRefusal::Grammar)?
                .0
            && text.len() == length.saturating_mul(2),
        CompactImportRefusal::Grammar,
    )?;
    require(
        text.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        CompactImportRefusal::Grammar,
    )?;
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            u8::from_str_radix(
                std::str::from_utf8(pair).map_err(|_| CompactImportRefusal::Grammar)?,
                16,
            )
            .map_err(|_| CompactImportRefusal::Grammar)
        })
        .collect()
}
struct Cursor<'a>(std::str::Lines<'a>);
impl<'a> Cursor<'a> {
    fn new(text: &'a str) -> Result<Self> {
        require(
            text.ends_with('\n') && !text.contains('\r'),
            CompactImportRefusal::Grammar,
        )?;
        Ok(Self(text.lines()))
    }
    fn exact(&mut self, text: &str) -> Result<()> {
        require(self.0.next() == Some(text), CompactImportRefusal::Grammar)
    }
    fn value(&mut self, field: &str) -> Result<&'a str> {
        self.0
            .next()
            .and_then(|line| line.strip_prefix(field))
            .and_then(|tail| tail.strip_prefix(' '))
            .ok_or(CompactImportRefusal::Grammar)
    }
    fn bytes(&mut self, field: &str) -> Result<Vec<u8>> {
        decode(self.value(field)?)
    }
    fn text(&mut self, field: &str) -> Result<String> {
        String::from_utf8(self.bytes(field)?).map_err(|_| CompactImportRefusal::Grammar)
    }
    fn done(&mut self) -> Result<()> {
        require(self.0.next().is_none(), CompactImportRefusal::Grammar)
    }
}

fn admit_files(capture: &str, timing: &str, manifest: &str, report: &str) -> Result<CompactCorpus> {
    let address_input = b"tripod/compact-run/address/v1\0candidate rule:phase7:wave0-rulings 128\npins FIXTURE_MERKLE_ROOT FIXTURE_REFERENCE_OUTPUT_KEY PINNED_PROGRAM\nroster compact-native-requirements-v1 48\ncontrols A2 A3 A4 N2 N3 N4 C2 C3 C4\nstaged 16\nceremony report\nmode record\nattempt 2\n";
    require(
        hash(address_input) == COMPACT_RUN_ADDRESS,
        CompactImportRefusal::Identity,
    )?;
    require(
        hash(manifest.as_bytes()) == COMPACT_MANIFEST_SHA256,
        CompactImportRefusal::Manifest,
    )?;
    let expected = format!(
        "{}  {COMPACT_RUN_ADDRESS}.report.capture\n{}  {COMPACT_RUN_ADDRESS}.report.capture.timing\n",
        hash(capture.as_bytes()),
        hash(timing.as_bytes())
    );
    require(manifest == expected, CompactImportRefusal::Manifest)?;
    require(
        hash(report.as_bytes()) == COMPACT_REPORT_SHA256,
        CompactImportRefusal::Report,
    )?;
    let mut report = Cursor::new(report)?;
    report.exact("run-report-schema compact-native-run-report 1")?;
    report.exact(&format!("run-address {COMPACT_RUN_ADDRESS}"))?;
    let source = [
        report.value("suite-commit")?.to_owned(),
        report.value("suite-tree")?.to_owned(),
    ];
    require(
        source.iter().all(|text| {
            text.len() == 40
                && text
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }),
        CompactImportRefusal::Identity,
    )?;
    for text in [
        "suite-clean yes",
        "mode record",
        "attempt 2",
        "expected-test-count 1",
        "expected-ceremony-count 1",
        "expected-setup-count 0",
        "expected-capture-count 1",
        "expected-timing-count 1",
        "expected-archive-file-count 4",
        "diagnostics-present yes",
        "expected-operation-count 92",
        "observed-operation-count 92",
    ] {
        report.exact(text)?;
    }
    let tip = report.value("elementsd-expected-tip")?.to_owned();
    report.exact("protocol-revision 8")?;
    report.exact("cargo-exit-code 0")?;
    report.exact(&format!("manifest-sha256 {COMPACT_MANIFEST_SHA256}"))?;
    report.exact("eligible yes")?;
    report.done()?;
    let mut clock = Cursor::new(timing)?;
    for line in [
        "timing-schema compact-phases-1",
        "ceremony-id report",
        "status passed",
    ] {
        clock.exact(line)?;
    }
    let mut durations = Vec::new();
    for field in ["phase-staging-ms", "phase-execution-ms", "total-ms"] {
        durations.push(
            clock
                .value(field)?
                .parse::<u64>()
                .map_err(|_| CompactImportRefusal::Grammar)?,
        );
    }
    require(
        durations[0]
            .checked_add(durations[1])
            .is_some_and(|sum| sum <= durations[2]),
        CompactImportRefusal::Grammar,
    )?;
    clock.done()?;
    let (operations, payload, deployment) = parse_capture(capture, &source, &tip)?;
    replay(&operations, &payload, deployment, source)
}

#[derive(Clone)]
struct Recorded {
    subject: OperationSubject,
    response: NativeOperationResponse,
}

#[allow(clippy::too_many_lines, reason = "closed ordered transcript grammar")]
fn parse_capture(
    text: &str,
    source: &[String; 2],
    tip: &str,
) -> Result<(Vec<Recorded>, String, [String; 3])> {
    let mut cursor = Cursor::new(text)?;
    cursor.exact("native-capture-schema 2")?;
    cursor.exact("ceremony-id report")?;
    require(
        cursor.text("rust-test-name")? == "compact_campaign_runs_against_the_pinned_target",
        CompactImportRefusal::Identity,
    )?;
    cursor.exact(&format!("suite-commit {}", source[0]))?;
    cursor.exact(&format!("suite-tree {}", source[1]))?;
    for line in [
        "fixture-digest-algorithm forward-v2",
        "run-id-input begin",
        "deployment-environment development",
    ] {
        cursor.exact(line)?;
    }
    let deployment = [
        cursor.text("deployment-network-id")?,
        cursor.text("deployment-genesis-id")?,
        cursor.text("deployment-target-contract")?,
    ];
    require(
        deployment[2] == "elements-tapscript-v2",
        CompactImportRefusal::Identity,
    )?;
    cursor.exact("handshake-protocol-schema 8")?;
    require(
        cursor.text("handshake-adapter-name")? == "elements-native-executor",
        CompactImportRefusal::Identity,
    )?;
    require(
        cursor.text("handshake-adapter-version")? == "2.1.0",
        CompactImportRefusal::Identity,
    )?;
    require(
        cursor.text("handshake-framework-revision")? == tip,
        CompactImportRefusal::Identity,
    )?;
    require(
        cursor.text("handshake-node-name")? == "Elements",
        CompactImportRefusal::Identity,
    )?;
    let version = cursor.text("handshake-node-version")?;
    let revision = cursor.text("handshake-binary-reported-revision")?;
    require(
        revision.len() >= 12 && tip.starts_with(&revision) && version.ends_with(&revision),
        CompactImportRefusal::Identity,
    )?;
    require(
        cursor.text("handshake-intended-executed-tip")? == tip,
        CompactImportRefusal::Identity,
    )?;
    for line in [
        "handshake-upstream-base 0 ",
        "handshake-topic-count 0",
        "environment-schema 8",
    ] {
        cursor.exact(line)?;
    }
    require(
        cursor.text("environment-chain")? == "elementsregtest",
        CompactImportRefusal::Identity,
    )?;
    require(
        cursor.text("environment-network-id")? == deployment[0]
            && cursor.text("environment-genesis-id")? == deployment[1],
        CompactImportRefusal::Identity,
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
    for (index, capability) in [
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
    ]
    .iter()
    .enumerate()
    {
        cursor.exact(&format!("environment-capability {index} {capability}"))?;
    }
    for line in [
        "environment-funding-count 0",
        "digest-count 0",
        "operation-count 92",
    ] {
        cursor.exact(line)?;
    }
    let mut subjects = Vec::new();
    let mut projections = Vec::new();
    for index in 0..92 {
        cursor.exact(&format!("operation {index} begin"))?;
        require(
            cursor.text("operation-id")? == format!("operation-{index}")
                && cursor.text("request-id")? == format!("request-{index}"),
            CompactImportRefusal::Identity,
        )?;
        cursor.exact("request-role auxiliary")?;
        let bytes = cursor.bytes("request-bytes")?;
        let subject: OperationSubject = serde_json::from_slice(&cursor.bytes("request-subject")?)
            .map_err(|_| CompactImportRefusal::Grammar)?;
        let submitted = match &subject {
            OperationSubject::Submission(s) => s.transaction_bytes.as_slice(),
            _ => &[],
        };
        require(bytes == submitted, CompactImportRefusal::Identity)?;
        for (field, expected) in [
            ("response-id", format!("response-{index}")),
            ("response-request-id", format!("request-{index}")),
            ("response-operation-id", format!("operation-{index}")),
        ] {
            require(
                cursor.text(field)? == expected,
                CompactImportRefusal::Identity,
            )?;
        }
        let verdict = cursor.value("response-verdict")?.to_owned();
        let layer = cursor.value("response-layer")?.to_owned();
        let identity = cursor.value("response-target-identity")?.to_owned();
        let detail = cursor.text("response-detail")?;
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
        subjects.push(subject);
        projections.push((verdict, layer, identity, detail));
    }
    let payload = cursor.text("legacy-rendering")?;
    cursor.exact("terminal-state complete")?;
    cursor.exact("run-id-input end")?;
    let prefix = text
        .split_once("capture-content-sha256 ")
        .ok_or(CompactImportRefusal::Grammar)?
        .0;
    require(
        cursor.value("capture-content-sha256")? == hash(prefix.as_bytes()),
        CompactImportRefusal::Identity,
    )?;
    cursor.exact("native-capture-end report")?;
    cursor.done()?;
    let mut body = Cursor::new(&payload)?;
    body.exact("compact-campaign-schema 1")?;
    body.exact(&format!("run-address {COMPACT_RUN_ADDRESS}"))?;
    let mut operations = Vec::new();
    for (subject, (verdict, layer, identity, detail)) in subjects.into_iter().zip(projections) {
        let response: NativeOperationResponse =
            serde_json::from_slice(&body.bytes("compact-validated-response")?)
                .map_err(|_| CompactImportRefusal::Grammar)?;
        require(
            response.schema == target_elements_conformance::protocol::NATIVE_PROTOCOL_SCHEMA,
            CompactImportRefusal::Identity,
        )?;
        response
            .validate_shape()
            .map_err(|_| CompactImportRefusal::Identity)?;
        let accepted = response.observed_layer == ObservedOutcomeLayer::Accepted;
        require(
            verdict == if accepted { "accepted" } else { "refused" },
            CompactImportRefusal::Identity,
        )?;
        require(
            layer
                == if accepted {
                    "accepted"
                } else {
                    "script-path-rejection"
                }
                && (accepted
                    || response.observed_layer == ObservedOutcomeLayer::ScriptPathRejection),
            CompactImportRefusal::Identity,
        )?;
        require(
            identity == response.accepted_txid.as_deref().unwrap_or("none")
                && detail == response.observed_detail.as_deref().unwrap_or(""),
            CompactImportRefusal::Identity,
        )?;
        operations.push(Recorded { subject, response });
    }
    Ok((operations, payload, deployment))
}

const fn amount(output: &TargetOutput) -> Result<u64> {
    if let ValueField::Explicit(n) = output.value() {
        Ok(n)
    } else {
        Err(CompactImportRefusal::Recipe)
    }
}
fn value(output: &TargetOutput, n: u64) -> TargetOutput {
    TargetOutput::new(
        output.asset(),
        ValueField::Explicit(n),
        output.nonce(),
        output.program().to_vec(),
    )
}

// Compare decoded coordinates without constructing a malformed transaction.
#[allow(
    clippy::too_many_lines,
    reason = "one exact coordinate comparison per byte recipe"
)]
fn verify_recipe(
    control: &[u8],
    bytes: &[u8],
    recipe: ByteRecipe,
    extra: Option<Outpoint>,
    ash: u8,
) -> Result<()> {
    let control = TargetTransaction::decode(control).map_err(|_| CompactImportRefusal::Recipe)?;
    let changed = TargetTransaction::decode(bytes).map_err(|_| CompactImportRefusal::Recipe)?;
    let mut inputs = control.inputs().to_vec();
    let mut outputs = control.outputs().to_vec();
    let mut witnesses = control.witnesses().to_vec();
    match recipe {
        ByteRecipe::OneInput => {
            inputs.remove(1);
            witnesses.remove(1);
            outputs[0] = value(
                &outputs[0],
                amount(&outputs[0])?
                    .checked_sub(100)
                    .ok_or(CompactImportRefusal::Recipe)?,
            );
        }
        ByteRecipe::FiveInputs => {
            inputs.insert(
                usize::from(ash),
                TargetInput::new(
                    extra.ok_or(CompactImportRefusal::Recipe)?,
                    inputs[0].sequence(),
                ),
            );
            witnesses.insert(usize::from(ash), witnesses[1].clone());
            outputs[0] = value(
                &outputs[0],
                amount(&outputs[0])?
                    .checked_add(100)
                    .ok_or(CompactImportRefusal::Recipe)?,
            );
        }
        ByteRecipe::RedirectSuccessor => {
            let o = &outputs[0];
            outputs[0] = TargetOutput::new(o.asset(), o.value(), o.nonce(), vec![0x51]);
        }
        ByteRecipe::SplitSuccessor | ByteRecipe::SplitChange => {
            let index = usize::from(recipe == ByteRecipe::SplitChange);
            let o = outputs[index].clone();
            let n = amount(&o)?;
            outputs[index] = value(&o, n / 2);
            outputs.insert(index + 1, value(&o, n - n / 2));
        }
        ByteRecipe::DivertUnit => {
            let o = outputs[0].clone();
            outputs[0] = value(
                &o,
                amount(&o)?
                    .checked_sub(1)
                    .ok_or(CompactImportRefusal::Recipe)?,
            );
            outputs.insert(
                1,
                TargetOutput::new(o.asset(), ValueField::Explicit(1), o.nonce(), vec![0x51]),
            );
        }
        ByteRecipe::MissingAuthorization => {
            *witnesses.last_mut().ok_or(CompactImportRefusal::Recipe)? =
                InputWitness::new(Vec::new());
        }
    }
    require(
        changed.encode() == bytes
            && changed.version() == control.version()
            && changed.lock_time() == control.lock_time()
            && changed.inputs() == inputs
            && changed.outputs() == outputs
            && changed.witnesses() == witnesses
            && changed != control,
        CompactImportRefusal::Recipe,
    )
}

#[allow(
    clippy::too_many_lines,
    reason = "settlement and the recorded findings are checked together"
)]
fn replay(
    operations: &[Recorded],
    payload: &str,
    deployment: [String; 3],
    source: [String; 2],
) -> Result<CompactCorpus> {
    require(operations.len() == 92, CompactImportRefusal::Census)?;
    let mut packets = VecDeque::new();
    for op in operations {
        if op.response.case.step.as_str().ends_with("/sign-mutant") {
            let OperationSubject::SponsorSigning(s) = &op.subject else {
                return Err(CompactImportRefusal::Recipe);
            };
            packets.push_back(s.finalized_transaction.clone());
        } else if op.response.case.step.as_str().ends_with("/submit-mutant") {
            let OperationSubject::Submission(s) = &op.subject else {
                return Err(CompactImportRefusal::Recipe);
            };
            let tx = TargetTransaction::decode(&s.transaction_bytes)
                .map_err(|_| CompactImportRefusal::Recipe)?;
            if tx.witnesses().last().is_some_and(|w| w.stack().is_empty())
                || !operations.iter().any(|other| {
                    other.response.case.step.as_str()
                        == op
                            .response
                            .case
                            .step
                            .as_str()
                            .replace("submit-mutant", "sign-mutant")
                })
            {
                packets.push_back(s.transaction_bytes.clone());
            }
        }
    }
    require(packets.len() == 16, CompactImportRefusal::Census)?;
    let mut planner = schedule::Planner::new(Box::new(move |control, recipe, extra, ash| {
        let bytes = packets.pop_front().ok_or("missing recipe packet")?;
        verify_recipe(control, &bytes, recipe, extra, ash).map_err(|e| format!("{e:?}"))?;
        Ok(bytes)
    }));
    let mut next = planner
        .next_step(None)
        .map_err(|_| CompactImportRefusal::Schedule)?;
    let mut observations = BTreeMap::new();
    let mut pending = None;
    for operation in operations {
        let step = next.take().ok_or(CompactImportRefusal::Schedule)?;
        require(
            step.subject() == &operation.subject && step.case() == &operation.response.case,
            CompactImportRefusal::Schedule,
        )?;
        if step.case().step.as_str().ends_with("/submit-mutant") {
            pending = Some((
                planner.jobs[planner.index]
                    .row
                    .clone()
                    .ok_or(CompactImportRefusal::Schedule)?,
                operation.response.observed_layer,
            ));
        }
        if step.case().step.as_str().ends_with("/submit-control")
            && let Some((row, observed)) = pending.take()
            && operation.response.observed_layer == ObservedOutcomeLayer::Accepted
            && crate::observed_boundary::matches_boundary(row.expected, observed)
        {
            observations.insert(
                row.manifest.requirement,
                NativeDisposition::Observed {
                    family: row.control,
                    expected: row.expected,
                    observed,
                },
            );
        }
        next = planner
            .next_step(Some((&operation.response.case, &operation.response)))
            .map_err(|_| CompactImportRefusal::Settlement)?;
    }
    require(
        next.is_none()
            && planner.suspended.is_empty()
            && planner.accepted_families.len() == 9
            && observations.len() == 16,
        CompactImportRefusal::Census,
    )?;
    let mut expected = format!("compact-campaign-schema 1\nrun-address {COMPACT_RUN_ADDRESS}\n");
    for operation in operations {
        let bytes =
            serde_json::to_vec(&operation.response).map_err(|_| CompactImportRefusal::Grammar)?;
        let _ = writeln!(
            expected,
            "compact-validated-response {} {}",
            bytes.len(),
            schedule::hex_bytes(&bytes)
        );
    }
    for row in &planner.rows {
        let _ = writeln!(
            expected,
            "roster {} {} {:?} expected {:?}",
            requirement_name(&row.manifest.requirement),
            row.control.name(),
            row.recipe,
            row.expected
        );
    }
    for finding in &planner.findings {
        let _ = writeln!(expected, "{finding}");
    }
    let _ = writeln!(
        expected,
        "suspended {:?}\nfailure {:?}",
        planner.suspended, planner.failure
    );
    require(expected == payload, CompactImportRefusal::Identity)?;
    for row in &planner.rows {
        observations
            .entry(row.manifest.requirement.clone())
            .or_insert_with(|| disposition_without_observation(row));
    }
    let standing = CandidateStanding::CurrentCandidate(CurrentCandidate::selected(
        deployment,
        BTreeSet::from([sha256(&planner.program)]),
        Some(source),
    ));
    Ok(CompactCorpus {
        standing,
        rows: observations,
        controls: planner.accepted_families,
    })
}

fn disposition_without_observation(row: &CampaignRow) -> NativeDisposition {
    if requirement_name(&row.manifest.requirement)
        == "Absent/SponsorIsolation/Sponsor/RuntimeCarrier/FocusedReject(MissingSponsorAuthorization)"
    {
        NativeDisposition::FormallyClosed(FormalClosure::NoSponsorAuthorizationCoordinate)
    } else {
        NativeDisposition::Unresolved(row.recipe.err().unwrap_or(UnresolvedRecipe::UnprobedRecipe))
    }
}

/// Admit the four immutable files once, including exact planner settlement.
///
/// # Errors
/// Returns the first failed archive or replay binding.
pub fn run_of_record() -> Result<&'static CompactCorpus> {
    static CORPUS: OnceLock<Result<CompactCorpus>> = OnceLock::new();
    CORPUS
        .get_or_init(|| admit_files(CAPTURE, TIMING, MANIFEST, REPORT))
        .as_ref()
        .map_err(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;
    use target_elements_conformance::protocol::{FundedOutput, WireOutpoint};
    use transaction::Txid;

    #[test]
    fn current_archive_replays_sixteen_rows_after_their_paired_controls() {
        let corpus = run_of_record().expect("admission");
        assert!(std::ptr::eq(corpus, run_of_record().expect("cached")));
        assert_eq!(corpus.controls.len(), 9);
        assert_eq!(corpus.rows.len(), 48);
        assert_eq!(
            corpus
                .rows
                .values()
                .filter(|d| matches!(d, NativeDisposition::Observed { .. }))
                .count(),
            16
        );
        assert_eq!(
            corpus
                .rows
                .values()
                .filter(|d| matches!(d, NativeDisposition::FormallyClosed(_)))
                .count(),
            1
        );
        assert_eq!(
            corpus
                .rows
                .values()
                .filter(|d| matches!(d, NativeDisposition::Unresolved(_)))
                .count(),
            31
        );
        let current = corpus.standing.current_candidate().expect("current");
        assert_eq!(
            current.pins,
            [
                target_elements_conformance::reference::FIXTURE_MERKLE_ROOT,
                target_elements_conformance::reference::FIXTURE_REFERENCE_OUTPUT_KEY,
                crate::bundle::PINNED_PROGRAM
            ]
        );
        assert_eq!(current.programs.len(), 1);
        assert!(corpus.standing.historical_candidate().is_none());
        let fixture = crate::bundle::fixture_bundle().expect("fixture");
        let plan = crate::plan::derive_evidence_plan(&fixture).expect("plan");
        assert_eq!(
            plan.current_campaign_census().expect("current census"),
            [16, 1, 31, 0]
        );
    }

    #[test]
    fn every_file_is_bound_before_replay() {
        for index in 0..4 {
            let mut files = [
                CAPTURE.to_owned(),
                TIMING.to_owned(),
                MANIFEST.to_owned(),
                REPORT.to_owned(),
            ];
            files[index].push('\n');
            assert!(matches!(
                admit_files(&files[0], &files[1], &files[2], &files[3]),
                Err(CompactImportRefusal::Manifest | CompactImportRefusal::Report)
            ));
        }
    }

    #[test]
    fn record_dispositions_bind_exactly_to_the_current_manifest() {
        let backlog = include_str!("../../../plans/backlog.md");
        let record = backlog
            .split_once("**Record (Wave 0 compact native campaign)**")
            .expect("record")
            .1
            .split_once("**Record (Wave 0 compact negative manifest)**")
            .expect("following record")
            .0;
        let corpus = run_of_record().expect("admission");
        let table = record
            .lines()
            .filter(|line| line.starts_with("| `Absent/") || line.starts_with("| `Present/"))
            .collect::<Vec<_>>();
        assert_eq!(table.len(), corpus.rows.len());
        for (key, disposition) in &corpus.rows {
            let prefix = format!("| `{}` | ", requirement_name(key));
            let rows = table
                .iter()
                .filter(|line| line.starts_with(&prefix))
                .collect::<Vec<_>>();
            assert_eq!(rows.len(), 1, "{key:?}");
            let row = rows[0];
            match disposition {
                NativeDisposition::Observed { family, .. } => {
                    assert!(row.contains(&format!("| {} | ScriptPathRejection | ScriptPathRejection | Observed; compact-run-of-record", family.name())));
                    assert!(row.contains(
                        "current_archive_replays_sixteen_rows_after_their_paired_controls"
                    ));
                }
                NativeDisposition::FormallyClosed(reason) => {
                    assert!(row.contains(&format!("Formally closed: `{reason:?}`")));
                    assert!(row.contains("no_sponsor_authorization_coordinate_covers_a2_a3_a4"));
                }
                NativeDisposition::Unresolved(reason) => {
                    assert!(row.contains(&format!("Unresolved: `{reason:?}`; no credit")));
                }
                NativeDisposition::Suspended(_) => {
                    panic!("the eligible archive suspends no family")
                }
            }
        }
    }

    fn parsed() -> (Vec<Recorded>, String, [String; 3], [String; 2]) {
        let mut report = Cursor::new(REPORT).expect("report");
        report
            .exact("run-report-schema compact-native-run-report 1")
            .expect("schema");
        report.value("run-address").expect("address");
        let source = [
            report.value("suite-commit").expect("source").to_owned(),
            report.value("suite-tree").expect("tree").to_owned(),
        ];
        let tip = REPORT
            .lines()
            .find_map(|line| line.strip_prefix("elementsd-expected-tip "))
            .expect("tip");
        let (ops, payload, deployment) = parse_capture(CAPTURE, &source, tip).expect("capture");
        (ops, payload, deployment, source)
    }

    #[test]
    fn recorded_operation_census_is_the_pinned_funding_and_signing_schedule() {
        let (ops, _, _, _) = parsed();
        let mut counts = [0; 6];
        for op in ops {
            let index = match op.subject {
                OperationSubject::Funding(ref subject) if subject.issue_asset => 0,
                OperationSubject::Funding(_) if op.response.case.step == "constructor" => 1,
                OperationSubject::SponsorFunding(_) => 2,
                OperationSubject::Funding(_) => 3,
                OperationSubject::SponsorSigning(_) => 4,
                OperationSubject::Submission(_) => 5,
                _ => panic!("unexpected schedule member"),
            };
            counts[index] += 1;
        }
        assert_eq!(counts, [1, 1, 1, 25, 23, 41]);
        assert_eq!(counts.iter().sum::<usize>(), 92);
    }

    #[test]
    fn closed_grammar_survives_rebinding_the_content_digest() {
        let (_, _, _, source) = parsed();
        let tip = REPORT
            .lines()
            .find_map(|line| line.strip_prefix("elementsd-expected-tip "))
            .expect("tip");
        for (before, after) in [
            ("handshake-protocol-schema 8", "handshake-protocol-schema 7"),
            ("operation-count 92", "operation-count 91"),
            ("request-role auxiliary", "request-role mutant"),
            (
                "response-request-id 9 726571756573742d30",
                "response-request-id 9 726571756573742d31",
            ),
            ("request-bytes 0 ", "request-bytes 1 "),
            (
                "environment-funding-count 0",
                "environment-funding-count 0\nunknown-field 0",
            ),
        ] {
            let changed = CAPTURE.replacen(before, after, 1);
            assert_ne!(changed, CAPTURE);
            let prefix = changed
                .split_once("capture-content-sha256 ")
                .expect("digest")
                .0;
            let rebound = format!(
                "{prefix}capture-content-sha256 {}\nnative-capture-end report\n",
                hash(prefix.as_bytes())
            );
            assert!(parse_capture(&rebound, &source, tip).is_err(), "{before}");
        }
    }

    #[test]
    fn rebound_subjects_signatures_readback_and_wrong_boundaries_cannot_gain_credit() {
        let (ops, payload, deployment, source) = parsed();
        for choice in 0..5 {
            let mut changed = ops.clone();
            match choice {
                0 => {
                    let op = changed
                        .iter_mut()
                        .find(|op| op.response.case.step.ends_with("/submit-mutant"))
                        .expect("mutant");
                    op.response.observed_layer =
                        ObservedOutcomeLayer::ConsensusRejectionBeforeScript;
                }
                1 => {
                    let op = changed
                        .iter_mut()
                        .find(|op| op.response.case.step.ends_with("/submit-control"))
                        .expect("control");
                    op.response.observed_layer = ObservedOutcomeLayer::ScriptPathRejection;
                }
                2 => {
                    let readback = changed
                        .iter_mut()
                        .find_map(|op| op.response.mined_readback.as_mut())
                        .expect("readback");
                    readback.raw_transaction[0] ^= 1;
                }
                3 => {
                    let op = changed
                        .iter_mut()
                        .find(|op| op.response.signature_bound_to.is_some())
                        .expect("signing");
                    op.response.signature_bound_to.as_mut().expect("bytes")[0] ^= 1;
                }
                _ => {
                    let op = changed
                        .iter_mut()
                        .find(|op| op.response.case.step.ends_with("/submit-mutant"))
                        .expect("mutant");
                    let OperationSubject::Submission(subject) = &mut op.subject else {
                        panic!("submission")
                    };
                    subject.transaction_bytes[0] ^= 1;
                }
            }
            assert!(
                replay(&changed, &payload, deployment.clone(), source.clone()).is_err(),
                "change {choice}"
            );
        }
    }

    #[test]
    fn no_sponsor_authorization_coordinate_covers_a2_a3_a4() {
        let mut planner =
            schedule::Planner::new(Box::new(|_, _, _, _| Err("no mutant is needed".into())));
        let bundle = crate::bundle::fixture_bundle().expect("bundle");
        let mut asset = bundle.closed_asset();
        asset.reverse();
        planner.bundle = Some(bundle);
        for index in 0..3 {
            planner.index = index;
            let family = planner.jobs[index].family;
            assert!(!family.sponsored);
            let coins = (0..family.ash)
                .map(|vout| FundedOutput {
                    outpoint: WireOutpoint {
                        txid: Txid::from_internal([0x42; 32]).to_target_display(),
                        vout: u32::from(vout),
                    },
                    asset: schedule::hex_bytes(&asset),
                    amount_satoshis: 100,
                    script: schedule::hex_bytes(&planner.program),
                })
                .collect::<Vec<_>>();
            planner.construct(&coins).expect("safe sponsorless control");
            let tx = TargetTransaction::decode(&planner.control).expect("control");
            assert_eq!(tx.inputs().len(), usize::from(family.ash));
            assert!(planner.active_sponsor.is_none());
            assert_eq!(planner.sponsors.len(), 0);
            for witness in tx.witnesses() {
                let stack = witness.stack();
                assert!(stack.len() >= 2);
                let script = &stack[stack.len() - 2];
                let control_block = &stack[stack.len() - 1];
                assert_ne!(script.len(), 0);
                assert!(control_block.len() >= 33);
                assert_eq!(control_block[0] & 0xfe, 196);
                // Removing this material removes an ASH script path, not sponsor authorization.
                assert_ne!(witness, &InputWitness::new(Vec::new()));
            }
            assert!(tx.inputs().get(usize::from(family.ash)).is_none());
            let mut witnesses = tx.witnesses().to_vec();
            witnesses[usize::from(family.ash) - 1] = InputWitness::new(Vec::new());
            let removed_ash_path = TargetTransaction::new(
                tx.version(),
                tx.inputs().to_vec(),
                tx.outputs().to_vec(),
                tx.lock_time(),
                witnesses,
            )
            .expect("test-local ASH path removal");
            assert_ne!(removed_ash_path.encode(), planner.control);
            assert_eq!(removed_ash_path.inputs(), tx.inputs());
            assert_eq!(
                removed_ash_path
                    .witnesses()
                    .last()
                    .expect("ASH member")
                    .stack()
                    .len(),
                0
            );
        }
        let closed = planner
            .rows
            .iter()
            .filter(|row| {
                matches!(
                    disposition_without_observation(row),
                    NativeDisposition::FormallyClosed(
                        FormalClosure::NoSponsorAuthorizationCoordinate
                    )
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(closed.len(), 1);
        assert_eq!(
            closed[0].manifest.mutation,
            compiler::operation_plan::RelationMutation::MissingSponsorAuthorization
        );
        assert!(!closed[0].control.sponsored);
    }
}
