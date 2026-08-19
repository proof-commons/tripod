//! The typed native-conformance report.
//!
//! # What a report is
//!
//! What one executor observed, for one exact reviewed contract and one
//! reviewed development binding. It changes no target fact: a
//! disagreement between a report and the contract is a finding to be
//! triaged, not an edit to either.
//!
//! # A raw report carries no assurance
//!
//! Everything in this module is a data-transfer object. A report is a
//! description of a run that someone produced, and describing a run is
//! not the same as the run having happened that way: a row can be
//! deleted, a status can be relabelled, a summary can be edited. The
//! assurance lives in [`crate::validate::ValidatedNativeConformanceReport`],
//! which is the only form the gate accepts and which recomputes every
//! field here from the fixtures, the plan, the claim registry, and the
//! transcript.
//!
//! # Every row carries its complete subject
//!
//! A case ordinal is local navigation inside one fixture census. It is
//! not semantic identity, and a report whose rows carried only ordinals
//! would describe a different run whenever a fixture changed underneath
//! it while continuing to look like the same report. Each row therefore
//! carries the complete fixture projection — script bytes, initial stack,
//! transaction context, enforcement layer, leaf version and status,
//! script source, expected outcome, expected resources, and the claims
//! the case bears on.
//!
//! # Deterministic by construction
//!
//! The same contract, binding, fixture census, claim registry, and
//! executor answers produce the same report bytes. Nothing here carries a
//! wall clock, an elapsed time, a hostname, a username, a process
//! identifier, a temporary path, the executor's path, or an environment
//! value, and every collection is ordered — cases by typed case identity,
//! evidence by requirement, claims by claim.
//!
//! # No identity
//!
//! There is no report digest and no field reserved for one. A report is
//! compared by its typed content and its exact bytes.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::claim::NativeEvidenceClaim;
use crate::fixture::{NativeCaseId, PrimitiveFixtureProjection};
use crate::protocol::{
    ExecutorCapability, NativeResourceObservation, NativeVerdict, ObservedFailureClass,
    RequestExpectationBoundary, WireEnvironment, WireExecutionDomain,
};

/// The report revision this harness writes.
///
/// Revision 2 binds complete fixture projections, typed evidence claims,
/// an observed executor environment, and separated provenance roles. A
/// revision-1 report describes a run whose harness established none of
/// those, so it stays what it was — historical evidence for its own exact
/// tree — rather than becoming a revision-2 report by reparsing.
pub const NATIVE_REPORT_SCHEMA: u32 = 2;

/// Which prototype census one report answers for.
///
/// A report may carry several roles only if each has its own exact case
/// and claim census `(´[PLAN-rule:guide10:report-roles]´)`. No report
/// this crate writes carries two: a primitive report answers the
/// primitive role, and each prototype report answers exactly one
/// relation, because the two matrices have separate case identities and
/// separate claim censuses and a report holding both would let one
/// relation's coverage read as the other's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PrototypeReportRole {
    /// The reviewed primitive census.
    PrimitiveConformance,
    /// The metadata-constructor continuity matrix.
    ConstructorContinuity,
    /// The wide-arithmetic floor matrix.
    WideFloor,
    /// An ad hoc primitive census a caller assembled.
    ///
    /// # Why the role is written into the report
    ///
    /// The type system already keeps this run away from the gate: an
    /// experimental run yields
    /// [`crate::validate::ExperimentalPrimitiveReport`], which no
    /// validator and no gate accepts. The role is recorded as well so
    /// that a *serialized* report says what it is. A published JSON
    /// document outlives the type that produced it, and a reader with the
    /// bytes alone must be able to see that the subject was a caller's
    /// choice.
    ExperimentalPrimitive,
    /// An ad hoc compound-prototype matrix a caller assembled.
    ExperimentalPrototype,
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

/// What the executor said it actually ran on.
///
/// Distinct from [`ActivationRecord`], which is what the caller intended.
/// A declaration and an observation are different kinds of statement, and
/// a report that stored one in the other's field would let a run label
/// one chain with another chain's identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedEnvironment {
    /// The class of deployment the executor observed.
    pub environment: WireEnvironment,
    /// The chain the node was configured to run.
    pub chain_name: String,
    /// The network identity of that chain.
    pub network_id: [u8; 32],
    /// The observed genesis identity of that chain.
    pub genesis_id: [u8; 32],
    /// The execution domains active there.
    pub active_domains: BTreeSet<WireExecutionDomain>,
    /// The leaf versions active there.
    pub active_leaf_versions: BTreeSet<u8>,
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
///
/// The provenance roles stay separated. What a binary says it is, what
/// the operator intended to run, what upstream base that derives from,
/// which local topics were folded in, and which adapter and framework
/// built the transactions are five different statements, and one revision
/// field answering all of them answers none of them (ADR-018).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutorProvenance {
    /// The protocol revision it spoke.
    pub protocol_schema: u32,

    /// What the adapter driving the node calls itself.
    pub adapter_name: String,
    /// What version that adapter calls itself.
    pub adapter_version: String,
    /// The transaction framework's revision, where it states one.
    pub framework_revision: Option<String>,

    /// What the executing node calls itself.
    pub node_name: String,
    /// The node's own version line.
    pub node_version: String,
    /// The revision the binary itself reports, where it embeds one.
    pub binary_reported_revision: Option<String>,

    /// The integration tip the operator intended to execute.
    pub intended_executed_tip: Option<String>,
    /// The upstream base that tip derives from.
    pub upstream_base: Option<String>,
    /// The local topic branches folded into that tip.
    pub included_local_topics: BTreeSet<String>,

    /// The domains it says it executes in.
    pub supported_domains: BTreeSet<WireExecutionDomain>,
    /// The leaf versions it says it accepts.
    pub supported_leaf_versions: BTreeSet<u8>,
    /// What it says its interface offers.
    pub capabilities: BTreeSet<ExecutorCapability>,
    /// What the caller declared it to be.
    pub declaration: ExecutorDeclaration,
}

impl ExecutorProvenance {
    /// Whether the run establishes the ADR-018 workspace provenance.
    ///
    /// A binary that reports no revision of its own, or a run that names
    /// no intended integration tip, has not established which program
    /// executed. The report says so rather than substituting a checkout's
    /// `HEAD`, which identifies intended source and never a binary.
    #[must_use]
    pub const fn establishes_workspace_provenance(&self) -> bool {
        self.binary_reported_revision.is_some()
            && self.intended_executed_tip.is_some()
            && self.upstream_base.is_some()
    }
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

/// One case's complete subject, observation, and verdict.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCaseResult {
    /// The complete fixture the executor was handed.
    pub fixture: PrimitiveFixtureProjection,
    /// The claims the case bears on.
    pub claims: BTreeSet<NativeEvidenceClaim>,
    /// What the executor reported.
    pub observed: ObservedNativeOutcome,
    /// How the two compared.
    pub status: CaseStatus,
}

impl NativeCaseResult {
    /// Which case this row answers for.
    #[must_use]
    pub const fn case(&self) -> NativeCaseId {
        self.fixture.case
    }
}

/// Where one evidence requirement sits in the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EvidencePlanClass {
    /// The gate requires this row to pass.
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

/// How one evidence requirement or claim came out.
///
/// `UnresolvedByDesign` is not success. It records that the project
/// deliberately did not attempt the row, which is a different statement
/// from having attempted it and succeeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EvidenceDisposition {
    /// Every case bearing on it passed, and there was at least one.
    Passed,
    /// A case bearing on it failed, or it is required and no passing
    /// case bears on it at all.
    Failed,
    /// It was deliberately not attempted.
    UnresolvedByDesign,
    /// A case bearing on it hit executor trouble.
    InfrastructureError,
}

/// One typed claim's requirement, bearing cases, and outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeEvidenceClaimResult {
    /// The claim.
    pub claim: NativeEvidenceClaim,
    /// The requirement's wire spelling.
    pub requirement: String,
    /// Whether a passing case must bear on it.
    pub required: bool,
    /// Why it is unresolved, where it is not required.
    pub unresolved_reason: Option<String>,
    /// The cases bearing on it, in canonical case order.
    pub bearing_cases: BTreeSet<NativeCaseId>,
    /// How it came out.
    pub disposition: EvidenceDisposition,
}

/// One evidence requirement's plan class and outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRequirementResult {
    /// The requirement's wire spelling.
    pub requirement: String,
    /// Where the plan puts it.
    pub plan: EvidencePlanClass,
    /// How it came out.
    pub disposition: EvidenceDisposition,
    /// How many cases bore on it.
    pub cases: u32,
    /// The claims it requires that no passing case bears on.
    pub missing_required_claims: BTreeSet<NativeEvidenceClaim>,
    /// The claims it owns that remain unresolved.
    pub unresolved_claims: BTreeSet<NativeEvidenceClaim>,
}

/// How complete the run was against the required plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReportCompleteness {
    /// Every required row passed and no claim remains unresolved.
    CompleteForPrimitivePlan,
    /// Every required row passed, and claims outside the required set
    /// remain deliberately unresolved.
    PartialUnresolvedClaims,
    /// A required row or claim did not pass, or a case failed.
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
    /// How many claims a passing case must bear on.
    pub required_claims_total: u32,
    /// How many of those a passing case bears on.
    pub required_claims_passed: u32,
    /// How many claims remain deliberately unresolved.
    pub claims_unresolved: u32,
    /// What the run adds up to.
    pub completeness: ReportCompleteness,
}

/// What one executor observed, for one contract and one binding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeConformanceReport {
    /// The report revision.
    pub schema: u32,
    /// Which prototype census this report answers for.
    pub role: PrototypeReportRole,
    /// The contract revision the run was stated against.
    pub target_contract_version: u32,
    /// Where the expected outcome sat in the exchange.
    pub expectation_boundary: RequestExpectationBoundary,
    /// The class of deployment the binding names.
    pub environment: WireEnvironment,
    /// The network the binding names.
    pub network_id: [u8; 32],
    /// The genesis identifier the binding names.
    pub genesis_id: [u8; 32],
    /// What the caller intended the environment to have active.
    pub activation: ActivationRecord,
    /// What the executor said it actually ran on.
    pub observed_environment: ObservedEnvironment,
    /// Which runner produced the observations.
    pub executor: ExecutorProvenance,
    /// The cases, in canonical case order.
    pub cases: Vec<NativeCaseResult>,
    /// The evidence rows, in requirement order.
    pub evidence: Vec<EvidenceRequirementResult>,
    /// The typed claims, in claim order.
    pub claims: Vec<NativeEvidenceClaimResult>,
    /// The counts and what they add up to.
    pub summary: NativeReportSummary,
}
