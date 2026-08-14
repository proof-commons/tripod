//! The typed native-conformance report.
//!
//! # What a report is
//!
//! What one executor observed, for one exact reviewed contract and one
//! development binding. It changes no target fact: a disagreement
//! between a report and the contract is a finding to be triaged, not an
//! edit to either.
//!
//! # Deterministic by construction
//!
//! The same contract, binding, fixture census, and executor answers
//! produce the same report bytes. Nothing here carries a wall clock, an
//! elapsed time, a hostname, a username, a process identifier, a
//! temporary path, the executor's path, or an environment value, and
//! every collection is ordered — cases by typed case identity, evidence
//! by requirement. Timing may appear only in noncanonical diagnostics,
//! never in the value compared for evidence.
//!
//! # No identity
//!
//! There is no report digest and no field reserved for one. A report is
//! compared by its typed content and its exact bytes.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::fixture::{ExpectedPrimitiveOutcome, NativeCaseId};
use crate::protocol::{
    ExecutorCapability, NativeResourceObservation, NativeVerdict, ObservedFailureClass,
    WireExecutionDomain,
};

/// The report revision this harness writes.
pub const NATIVE_REPORT_SCHEMA: u32 = 1;

/// Which class of deployment the run was bound to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WireEnvironment {
    /// A network under the project's own control.
    Development,
}

/// What the caller intended the environment to have active.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivationRecord {
    /// Whether the reviewed domain was expected active.
    pub tapscript_expected_active: bool,
    /// The leaf version the caller expected to be usable.
    pub required_leaf_version: u8,
    /// The capabilities the caller intended to rely upon.
    pub required_capabilities: BTreeSet<String>,
}

/// What the caller declared the executor to be.
///
/// A declaration, recorded as provenance. It proves nothing about the
/// program: a mock declared reviewed is still a mock, and this field
/// records only what was claimed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExecutorDeclaration {
    /// Declared a mock. Such a run can never satisfy the gate.
    Mock,
    /// Declared the reviewed nonmock runner.
    ReviewedNonMock,
}

/// What the executor said about itself, plus what it was declared to be.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutorProvenance {
    /// The protocol revision it spoke.
    pub protocol_schema: u32,
    /// What it calls itself.
    pub implementation_name: String,
    /// What version it calls itself.
    pub implementation_version: String,
    /// The upstream revision it states, where it states one.
    pub upstream_revision: Option<String>,
    /// The domains it says it executes in.
    pub supported_domains: BTreeSet<WireExecutionDomain>,
    /// The leaf versions it says it accepts.
    pub supported_leaf_versions: BTreeSet<u8>,
    /// What it says its interface offers.
    pub capabilities: BTreeSet<ExecutorCapability>,
    /// What the caller declared it to be.
    pub declaration: ExecutorDeclaration,
}

/// What the executor observed for one case.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedNativeOutcome {
    /// What the target did.
    pub verdict: NativeVerdict,
    /// The final main stack, where one was reported.
    pub final_stack: Option<Vec<Vec<u8>>>,
    /// The final alternate stack, where one was reported.
    pub final_altstack: Option<Vec<Vec<u8>>>,
    /// The failure class, where one was reported.
    pub observed_failure: Option<ObservedFailureClass>,
    /// What the execution cost.
    pub resources: NativeResourceObservation,
}

/// How one case came out.
///
/// There is no `skipped`. A case the run did not establish is a case the
/// run did not establish, and giving it a status that reads like a mild
/// pass is how an incomplete suite looks green.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CaseStatus {
    /// The observation matched what the contract requires.
    Passed,
    /// The observation did not match.
    Failed,
    /// The executor could not run the case.
    InfrastructureError,
}

/// One case's expectation, observation, and verdict.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCaseResult {
    /// Which case.
    pub case: NativeCaseId,
    /// What the reviewed contract requires.
    pub expected: ExpectedPrimitiveOutcome,
    /// What the executor reported.
    pub observed: ObservedNativeOutcome,
    /// How the two compared.
    pub status: CaseStatus,
}

/// Where one evidence requirement sits in the Guide-9 plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EvidencePlanClass {
    /// The Guide-9 gate requires this row to pass.
    Required,
    /// The static contract does not currently establish enough for this
    /// row to be answerable.
    UnresolvedByDesign,
    /// The reviewed static contract marks the underlying capability
    /// unsupported.
    UnsupportedByStaticContract,
    /// Answering this row needs complete protocol transaction evidence,
    /// which is later work.
    DeferredToTransactionEvidence,
}

/// How one evidence requirement came out.
///
/// `UnresolvedByDesign` is not success. It records that the project
/// deliberately did not attempt the row, which is a different statement
/// from having attempted it and succeeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EvidenceDisposition {
    /// Every case bearing on the row passed, and there was at least one.
    Passed,
    /// A case bearing on the row failed, or the row is required and no
    /// case bears on it at all.
    Failed,
    /// The row was deliberately not attempted.
    UnresolvedByDesign,
    /// A case bearing on the row hit executor trouble.
    InfrastructureError,
}

/// One evidence requirement's plan class and outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRequirementResult {
    /// The requirement's wire spelling.
    pub requirement: String,
    /// Where the Guide-9 plan puts it.
    pub plan: EvidencePlanClass,
    /// How it came out.
    pub disposition: EvidenceDisposition,
    /// How many cases bore on it.
    pub cases: u32,
}

/// How complete the run was against the Guide-9 required plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReportCompleteness {
    /// Every required row passed and nothing remains unresolved.
    CompleteForRequiredPlan,
    /// Every required row passed, but rows outside the required plan
    /// remain deliberately unresolved.
    PartialUnresolvedRemains,
    /// A required row did not pass, or a case failed.
    Failed,
}

/// The counts, and what they add up to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReportSummary {
    /// How many cases ran.
    pub cases_total: u32,
    /// How many passed.
    pub cases_passed: u32,
    /// How many failed.
    pub cases_failed: u32,
    /// How many hit executor trouble.
    pub cases_infrastructure_error: u32,
    /// How many rows the required plan holds.
    pub required_evidence_total: u32,
    /// How many of those passed.
    pub required_evidence_passed: u32,
    /// How many rows remain deliberately unresolved.
    pub evidence_unresolved_by_design: u32,
    /// What the run adds up to.
    pub completeness: ReportCompleteness,
}

/// What one executor observed, for one contract and one binding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeConformanceReport {
    /// The report revision.
    pub schema: u32,
    /// The contract revision the run was stated against.
    pub target_contract_version: u32,
    /// The class of deployment.
    pub environment: WireEnvironment,
    /// The network the run was bound to.
    pub network_id: [u8; 32],
    /// The network's genesis identifier.
    pub genesis_id: [u8; 32],
    /// What the caller intended the environment to have active.
    pub activation: ActivationRecord,
    /// Which runner produced the observations.
    pub executor: ExecutorProvenance,
    /// The cases, in canonical case order.
    pub cases: Vec<NativeCaseResult>,
    /// The evidence rows, in requirement order.
    pub evidence: Vec<EvidenceRequirementResult>,
    /// The counts and what they add up to.
    pub summary: NativeReportSummary,
}
