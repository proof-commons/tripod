//! The typed compound-prototype conformance report.
//!
//! # A separate report, for the same reason the fixtures are separate
//!
//! A primitive report answers what one reviewed primitive did. A
//! prototype report answers whether a multi-step construction held
//! together across a whole target output. The two have different case
//! identities, different claim censuses, and different completeness
//! statements, and one report type carrying both would produce rows a
//! reader could not tell apart — which is exactly the outcome the
//! separate fixture language exists to prevent
//! `(´[PLAN-rule:guide10:compound-fixture]´)` and
//! `(´[PLAN-rule:guide10:report-roles]´)`.
//!
//! # One relation per report
//!
//! Each report answers exactly one [`PrototypeRelation`]. The role and
//! the relation are both recorded and are checked against each other, so
//! a report cannot carry the constructor role over wide-floor rows and
//! have the mismatch pass as a labelling detail.
//!
//! # A raw report carries no assurance
//!
//! Everything here is a data-transfer object, exactly as in
//! [`crate::report`]. A report is a description of a run that someone
//! produced, and describing a run is not the run having happened that
//! way. The assurance lives in
//! [`crate::prototype_validate::ValidatedPrototypeReport`], which
//! recomputes every field below from the matrix, the claim census, and
//! the transcript.
//!
//! # Every row carries its complete subject
//!
//! A case name is a label for a reader, not an identity anything
//! persists `(´[PLAN-rule:guide10:fixture-binding]´)`. Each row
//! therefore carries the complete fixture projection: the exact script,
//! the exact initial witness stack, the complete stated construction —
//! internal key, tree, executing leaf, control block, predecessor
//! program, and required outputs — the enforcement layer, the execution
//! domain, the leaf version and its reviewed status, the script source,
//! the expected verdict, the expected resources, and the claims the case
//! bears on.
//!
//! # Deterministic by construction
//!
//! The same contract, binding, matrix, claim census, and executor
//! answers produce the same report bytes. Nothing here carries a wall
//! clock, an elapsed time, a hostname, a username, a process identifier,
//! a temporary path, the executor's path, or an environment value, and
//! every collection is ordered — cases in the matrix's own canonical
//! order, claims by claim, bearing cases by case identity
//! `(´[PLAN-rule:guide10:report-determinism]´)`.
//!
//! # No identity
//!
//! There is no report digest and no field reserved for one
//! `(´[PLAN-rule:guide10:report-no-digest]´)`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::fixture::{
    EnforcementLayer, ExpectedResourceObservation, FixtureScriptSource, LeafVersionStatus,
};
use crate::protocol::{RequestExpectationBoundary, WireEnvironment, WireExecutionDomain};
use crate::prototype::{
    ExpectedPrototypeOutcome, PrototypeCaseId, PrototypeClaim, PrototypeConstruction,
    PrototypeRelation,
};
use crate::report::{
    ActivationRecord, CaseStatus, EvidenceDisposition, ExecutorProvenance, ObservedEnvironment,
    ObservedNativeOutcome, PrototypeReportRole,
};

/// The prototype-report revision this harness writes.
///
/// Revision 1 because this is the first revision of *this* report. It is
/// deliberately not numbered to match the primitive report's revision 2:
/// the two are different documents with different rows, and giving them
/// one number would invite a reader to assume a shared history they do
/// not have.
pub const PROTOTYPE_REPORT_SCHEMA: u32 = 1;

/// One compound fixture's complete subject, as a report states it.
///
/// The compound fixture already carries most of this; the remaining
/// fields are the run-level facts a fixture is stated *against* rather
/// than facts it states — the bound network and genesis, the execution
/// domain, and the enforcement layer — which a row must carry for the
/// same reason a primitive row does: without them the row describes a
/// spend without saying on which chain, in which domain, or under whose
/// rules the verdict was reached.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrototypeFixtureProjection {
    /// Which case.
    pub case: PrototypeCaseId,
    /// The contract revision the fixture was stated against.
    pub target_contract_version: u32,
    /// The network the run was bound to.
    pub network_id: [u8; 32],
    /// The genesis identifier the run was bound to.
    pub genesis_id: [u8; 32],
    /// The execution domain.
    pub execution_domain: WireExecutionDomain,
    /// The executing leaf's version byte.
    pub leaf_version: u8,
    /// Whether that byte is the reviewed one.
    ///
    /// Always [`LeafVersionStatus::Reviewed`] for a coherent compound
    /// fixture — one stating any other byte is refused by
    /// [`crate::prototype::CompoundPrototypeFixture::defect`] — and
    /// carried anyway, because a row that omitted it would be a row
    /// whose reader had to know that rule to read it.
    pub leaf_version_status: LeafVersionStatus,
    /// Which rule the stated verdict belongs to.
    ///
    /// Always [`EnforcementLayer::Consensus`]. A compound relation states
    /// that a spend stands or does not, and standardness is not that
    /// contract: a relay refusal leaves the spend valid, which is not a
    /// statement any claim here is about.
    pub enforcement_layer: EnforcementLayer,
    /// Where the exact script bytes came from.
    ///
    /// Always [`FixtureScriptSource::TypedProgram`]. Both matrices emit
    /// their scripts from the typed prototype programs this crate builds;
    /// neither states raw bytes, because a compound relation has no
    /// malformed-encoding row to state them for.
    pub script_source: FixtureScriptSource,
    /// The exact script the executing leaf runs.
    pub script: Vec<u8>,
    /// The exact initial witness stack, deepest item first.
    pub initial_stack: Vec<Vec<u8>>,
    /// The complete construction the executor was required to
    /// materialize.
    pub construction: PrototypeConstruction,
    /// What the target was required to do with the spend.
    pub expected: ExpectedPrototypeOutcome,
    /// What the execution was expected to cost.
    pub expected_resources: ExpectedResourceObservation,
    /// The typed claims the case bears on.
    pub claims: BTreeSet<PrototypeClaim>,
}

/// One case's complete subject, observation, and verdict.
///
/// The observation type is the primitive report's
/// ([`ObservedNativeOutcome`]) and is shared rather than restated: an
/// executor observes the same things about a compound spend as about a
/// primitive one, and two copies of that shape would eventually disagree
/// about what an executor is able to say.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrototypeCaseResult {
    /// The complete fixture the executor was handed.
    pub fixture: PrototypeFixtureProjection,
    /// The claims the case bears on.
    pub claims: BTreeSet<PrototypeClaim>,
    /// What the executor reported.
    pub observed: ObservedNativeOutcome,
    /// How the two compared.
    pub status: CaseStatus,
}

impl PrototypeCaseResult {
    /// Which case this row answers for.
    #[must_use]
    pub fn case(&self) -> PrototypeCaseId {
        self.fixture.case.clone()
    }
}

/// One typed prototype claim's requirement, bearing cases, and outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrototypeClaimResult {
    /// The claim.
    pub claim: PrototypeClaim,
    /// The relation that owns it.
    pub relation: PrototypeRelation,
    /// Whether a passing case must bear on it.
    pub required: bool,
    /// Why it is unresolved, where it is not required.
    pub unresolved_reason: Option<String>,
    /// The cases bearing on it, in canonical case order.
    pub bearing_cases: BTreeSet<PrototypeCaseId>,
    /// How it came out.
    pub disposition: EvidenceDisposition,
}

/// How complete one prototype run was against its relation's claims.
///
/// # Why the primitive completeness is not among these
///
/// [`crate::report::ReportCompleteness::CompleteForPrimitivePlan`]
/// answers a question this report never asks. A prototype report has no
/// evidence plan and no primitive census, so a value naming one would be
/// representable and meaningless — and the whole reason the two reports
/// are separate types is that a meaningless value in a report is one a
/// reader has to already know to discount
/// `(´[PLAN-rule:guide10:report-completeness]´)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PrototypeReportCompleteness {
    /// Every constructor-continuity claim has a passing bearing case,
    /// and every case passed.
    CompleteForConstructorPrototype,
    /// Every wide-floor claim has a passing bearing case, and every case
    /// passed.
    CompleteForWideFloorPrototype,
    /// Every case passed, and a claim outside the required set remains
    /// deliberately unresolved.
    PartialUnresolvedClaims,
    /// A case failed, a case hit executor trouble, or a required claim
    /// has no passing case bearing on it.
    Failed,
}

/// The counts, and what they add up to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrototypeReportSummary {
    /// How many cases ran.
    pub cases_total: u32,
    /// How many passed.
    pub cases_passed: u32,
    /// How many failed.
    pub cases_failed: u32,
    /// How many hit executor trouble.
    pub cases_infrastructure_error: u32,
    /// How many claims the relation holds.
    pub claims_total: u32,
    /// How many claims a passing case must bear on.
    pub required_claims_total: u32,
    /// How many of those a passing case bears on.
    pub required_claims_passed: u32,
    /// How many claims remain deliberately unresolved.
    pub claims_unresolved: u32,
    /// What the run adds up to.
    pub completeness: PrototypeReportCompleteness,
}

/// What one executor observed for one prototype relation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrototypeConformanceReport {
    /// The report revision.
    pub schema: u32,
    /// Which prototype census this report answers for.
    pub role: PrototypeReportRole,
    /// The relation whose matrix ran.
    ///
    /// Recorded alongside the role and checked against it. One is the
    /// report's own classification and the other is the matrix's, and a
    /// run whose two disagree has mislabelled one of them.
    pub relation: PrototypeRelation,
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
    /// The cases, in the matrix's canonical order.
    pub cases: Vec<PrototypeCaseResult>,
    /// The typed claims, in claim order.
    pub claims: Vec<PrototypeClaimResult>,
    /// The counts and what they add up to.
    pub summary: PrototypeReportSummary,
}
