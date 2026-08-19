//! The compound-prototype comparison, report validator, and gate.
//!
//! # The comparison compares
//!
//! The fixture's stated expectation and the executor's reported
//! observation are two independent values, and a case passes only when
//! they agree. A mismatch is a finding to triage against its four
//! possible owners — the fixture, the adapter, the prototype program, or
//! the target — and never a reason to adopt the observation as the new
//! expectation `(´[PLAN-rule:guide10:compare-verdict]´)`.
//!
//! # What a compound fixture does and does not fix
//!
//! A compound fixture states a spend verdict, not a failure class. That
//! is a property of the subject rather than an omission: the relations
//! here are about whether a construction holds together, and the reason
//! a target gives for refusing a spend it was always going to refuse is
//! an observation worth recording and not a requirement worth failing an
//! honest executor over. The observed class is therefore carried into
//! every row and compared against nothing
//! `(´[PLAN-rule:guide10:compare-failure]´)`.
//!
//! The stacks are the same story from the other side. A validating node
//! exposes no interpreter stack, so both prototype programs reduce their
//! expected intermediate state to one final truth item and let the
//! consensus verdict carry the shape — which is the route Guide-10
//! admits for exactly this case `(´[PLAN-rule:guide10:compare-stack]´)`.
//! Where an executor does report a stack it is recorded; nothing here
//! invents an expectation to compare it against.
//!
//! # A broad claim is not established by one case
//!
//! Beneath the relation sits the claim census
//! ([`crate::prototype::PrototypeClaim`]). A claim passes only when a
//! case bearing on it passed and none bearing on it failed, and a claim
//! no case bears on is recorded with its reason rather than absorbed
//! `(´[PLAN-rule:guide10:claim-result]´)`.
//!
//! # The gate is not the report
//!
//! The gate reads a [`ValidatedPrototypeReport`], which exists only once
//! every field of the raw report has been recomputed from the matrix,
//! the claim census, and the transcript, and found equal in both
//! directions. Removing a failed row, relabelling a claim, clearing the
//! claim array, duplicating a passing row, or editing the summary all
//! fail there rather than passing here.

use std::collections::{BTreeMap, BTreeSet};

use tapscript::TapscriptProgram;
use target_elements::{ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition};

use crate::claim::ClaimRequirement;
use crate::error::NativeConformanceError;
use crate::executor::ExecutionTranscript;
use crate::fixture::{EnforcementLayer, FixtureScriptSource, LeafVersionStatus};
use crate::protocol::{
    NativeVerdict, RequestExpectationBoundary, WireEnvironment, WireExecutionDomain,
};
use crate::prototype::{
    CanonicalPrototypeMatrix, CompoundPrototypeFixture, ExpectedPrototypeOutcome, PrototypeCaseId,
    PrototypeClaim, PrototypeRelation, constructor_case_matrix, wide_floor_case_matrix,
};
use crate::prototype_report::{
    PROTOTYPE_REPORT_SCHEMA, PrototypeCaseResult, PrototypeClaimResult, PrototypeConformanceReport,
    PrototypeFixtureProjection, PrototypeReportCompleteness, PrototypeReportSummary,
};
use crate::report::{
    CaseStatus, EvidenceDisposition, ExecutorDeclaration, ObservedEnvironment,
    ObservedNativeOutcome, PrototypeReportRole,
};

impl PrototypeRelation {
    /// The report role that answers for this relation.
    ///
    /// Stated rather than assumed, so a report whose role and relation
    /// disagree is a validation failure rather than a labelling detail:
    /// the role is what a reader indexes coverage by, and a wide-floor
    /// matrix filed under the constructor role would read as constructor
    /// coverage that nothing established.
    #[must_use]
    pub const fn report_role(self) -> PrototypeReportRole {
        match self {
            Self::MetadataConstructorContinuity => PrototypeReportRole::ConstructorContinuity,
            Self::WideFloorRelation => PrototypeReportRole::WideFloor,
        }
    }
}

/// Builds the report of one canonical prototype run.
///
/// This is the prototype evidence path. It accepts only a
/// [`CanonicalPrototypeMatrix`], and it does not take that wrapper's word
/// for it: the matrix is regenerated for the run's relation and compared
/// row for row, in order, including each row's claim set
///.
///
/// # Why coherence was never enough
///
/// [`CompoundPrototypeFixture::defect`] establishes that a row's tree
/// contains its executing leaf, that its predecessor program is the one
/// its own tree determines, and that its claims belong to its own
/// relation. Every one of those holds for a bare leaf whose script pushes
/// a true literal and which carries all eleven wide-floor claims. The
/// node accepts the true script, the claims are copied from the fixture,
/// and the run reads as a complete relation. What the row does *not*
/// establish is that it is one of the relation's canonical cases, and
/// that is the question asked here.
///
/// # Errors
///
/// [`NativeConformanceError::CanonicalPrototypeMatrixUnavailable`] when
/// the canonical matrix cannot be regenerated,
/// [`NativeConformanceError::NoncanonicalPrototypeMatrix`] or
/// [`NativeConformanceError::NoncanonicalPrototypeCase`] when the offered
/// matrix is not that one, and then every error
/// [`evaluate_experimental_prototypes`] states.
pub fn evaluate_prototypes(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    matrix: CanonicalPrototypeMatrix<'_>,
    transcript: &ExecutionTranscript,
) -> Result<PrototypeConformanceReport, NativeConformanceError> {
    let relation = matrix.relation();
    let regenerated = regenerate(target, relation)?;
    let offered = matrix.rows();
    if offered.len() != regenerated.len() {
        return Err(NativeConformanceError::NoncanonicalPrototypeMatrix);
    }
    // Row for row and in order. The order is part of the subject: the
    // report's case order is the matrix's own, so a permutation is a
    // different matrix rather than the same one rearranged.
    for (row, canonical) in offered.iter().zip(&regenerated) {
        if row != canonical {
            return Err(NativeConformanceError::NoncanonicalPrototypeCase(
                canonical.case.clone(),
            ));
        }
    }

    evaluate_matrix(
        target,
        binding,
        relation,
        offered,
        transcript,
        relation.report_role(),
    )
}

/// The canonical matrix of one relation, as rows.
fn regenerate(
    target: &ReviewedElementsTapscriptDefinition,
    relation: PrototypeRelation,
) -> Result<Vec<CompoundPrototypeFixture>, NativeConformanceError> {
    match relation {
        PrototypeRelation::MetadataConstructorContinuity => constructor_case_matrix(target)
            .map(|matrix| matrix.rows().to_vec())
            .map_err(|_| NativeConformanceError::CanonicalPrototypeMatrixUnavailable),
        PrototypeRelation::WideFloorRelation => wide_floor_case_matrix(target)
            .map(|matrix| matrix.rows().to_vec())
            .map_err(|_| NativeConformanceError::CanonicalPrototypeMatrixUnavailable),
    }
}

/// Builds the report of one ad hoc prototype run.
///
/// An arbitrary compound matrix may still be executed and described. What
/// it may not do is become evidence: the result is an
/// [`ExperimentalPrototypeReport`], which no validator and no gate
/// accepts, and whose recorded role says so in the serialized document as
/// well as in the type.
///
/// # Errors
///
/// [`NativeConformanceError::TargetContractMismatch`] when the binding is
/// not welded to this contract or a fixture is stated against another
/// revision, [`NativeConformanceError::PrototypeMatrixRelationMismatch`]
/// when a row does not belong to the relation being run,
/// [`NativeConformanceError::DuplicatePrototypeCase`] when two rows
/// declare one case,
/// [`NativeConformanceError::IncoherentPrototypeFixture`] when a row does
/// not state a coherent case, and
/// [`NativeConformanceError::MissingPrototypeResponse`] when the
/// transcript does not answer a row.
pub fn evaluate_experimental_prototypes(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    relation: PrototypeRelation,
    matrix: &[CompoundPrototypeFixture],
    transcript: &ExecutionTranscript,
) -> Result<ExperimentalPrototypeReport, NativeConformanceError> {
    Ok(ExperimentalPrototypeReport {
        report: evaluate_matrix(
            target,
            binding,
            relation,
            matrix,
            transcript,
            PrototypeReportRole::ExperimentalPrototype,
        )?,
    })
}

/// The report of one run over one matrix, under a stated role.
fn evaluate_matrix(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    relation: PrototypeRelation,
    matrix: &[CompoundPrototypeFixture],
    transcript: &ExecutionTranscript,
    role: PrototypeReportRole,
) -> Result<PrototypeConformanceReport, NativeConformanceError> {
    let definition = target.definition();
    let domain = WireExecutionDomain::of(definition.execution_domain())
        .ok_or(NativeConformanceError::TargetContractMismatch)?;
    // The binding must be the one this exact contract validated. A
    // same-revision neighbour is a different contract, and a run stated
    // against one and reported against the other would attach this
    // contract's name to that contract's evidence.
    if !binding.welded_to(target) {
        return Err(NativeConformanceError::TargetContractMismatch);
    }

    // The same three welds the primitive path applies, for the same
    // reason: a transcript is bound to the contract, the binding, and the
    // subjects it was produced under
    // (´[PLAN-rule:guide11:transcript-binding]´).
    if transcript.target() != &target.projection() {
        return Err(NativeConformanceError::TranscriptTargetRebinding);
    }
    if transcript.deployment() != &binding.projection() {
        return Err(NativeConformanceError::TranscriptDeploymentRebinding);
    }
    crate::executor::compare_environment(target, binding, transcript.environment())?;
    if let Some(case) = transcript
        .prototype_responses()
        .keys()
        .find(|case| !transcript.prototype_requests().contains_key(case))
    {
        return Err(NativeConformanceError::UnrequestedPrototypeResponse(
            case.clone(),
        ));
    }

    let mut cases = Vec::new();
    let mut seen: BTreeSet<PrototypeCaseId> = BTreeSet::new();
    let mut per_claim: BTreeMap<PrototypeClaim, Vec<(PrototypeCaseId, CaseStatus)>> =
        BTreeMap::new();

    for fixture in matrix {
        if fixture.case.relation != relation {
            return Err(NativeConformanceError::PrototypeMatrixRelationMismatch {
                case: fixture.case.clone(),
            });
        }
        if !seen.insert(fixture.case.clone()) {
            return Err(NativeConformanceError::DuplicatePrototypeCase(
                fixture.case.clone(),
            ));
        }
        // A fixture that does not state a coherent case has no report
        // subject. Checking it here rather than trusting the census is
        // what makes the report's rows answerable: an incoherent row
        // could be satisfied by a transaction other than the one it
        // means, and an executor's agreement with it would establish
        // nothing (´[PLAN-rule:guide10:fixture-validation]´).
        if let Some(defect) = fixture.defect(target) {
            return Err(NativeConformanceError::IncoherentPrototypeFixture {
                case: fixture.case.clone(),
                defect: Box::new(defect),
            });
        }

        // The construction, the program, and the witness stack being
        // reported must be the ones that case was executed with. A
        // substituted construction is the sharpest of these: a row's
        // whole claim is that one exact tree held together, and a report
        // naming a tree the executor never built would credit the
        // relation to a construction nobody ran.
        let requested = transcript
            .prototype_requests()
            .get(&fixture.case)
            .ok_or_else(|| NativeConformanceError::MissingPrototypeRequest(fixture.case.clone()))?;
        if requested != &fixture.subject() {
            return Err(NativeConformanceError::PrototypeTranscriptSubjectMismatch(
                fixture.case.clone(),
            ));
        }
        let response = transcript
            .prototype_responses()
            .get(&fixture.case)
            .ok_or_else(|| {
                NativeConformanceError::MissingPrototypeResponse(fixture.case.clone())
            })?;

        let observed = ObservedNativeOutcome {
            verdict: response.verdict,
            final_stack: response.final_stack.clone(),
            final_altstack: response.final_altstack.clone(),
            observed_failure: response.observed_failure,
            resources: response.resources,
        };
        let status = compare(fixture, &observed);

        for claim in &fixture.claims {
            per_claim
                .entry(*claim)
                .or_default()
                .push((fixture.case.clone(), status));
        }

        cases.push(PrototypeCaseResult {
            fixture: projection(target, fixture, binding, domain),
            claims: fixture.claims.clone(),
            observed,
            status,
        });
    }

    let claims = claim_rows(relation, &per_claim);
    let summary = summarize(relation, &cases, &claims);
    let observation = transcript.environment();

    Ok(PrototypeConformanceReport {
        schema: PROTOTYPE_REPORT_SCHEMA,
        role,
        relation,
        target_contract_version: definition.version().get(),
        expectation_boundary: RequestExpectationBoundary::ExecutorReceivesSubjectOnly,
        environment: WireEnvironment::Development,
        network_id: binding.binding().network_id(),
        genesis_id: binding.binding().genesis_id(),
        activation: crate::validate::activation_record_of(binding),
        observed_environment: ObservedEnvironment {
            environment: observation.environment,
            chain_name: observation.chain_name.clone(),
            network_id: observation.network_id,
            genesis_id: observation.genesis_id,
            active_domains: observation.active_domains.clone(),
            active_leaf_versions: observation.active_leaf_versions.clone(),
        },
        executor: crate::validate::provenance_of(transcript),
        cases,
        claims,
        summary,
    })
}

/// Where one compound row's script bytes actually came from.
///
/// # The provenance is established, not asserted
///
/// Every compound row used to be stamped
/// [`FixtureScriptSource::TypedProgram`] unconditionally, though
/// `CompoundPrototypeFixture::script` is a public byte vector and nothing
/// in the coherence check establishes that the bytes came from a typed
/// program. A row could therefore carry bytes no typed program encodes —
/// a lone push prefix with no payload, say — and the report would call
/// them a typed program's own encoding.
///
/// So the claim is proved instead of stated. The bytes are decoded
/// through the reviewed contract and re-encoded, and the source is
/// `TypedProgram` exactly when the round trip returns the original bytes:
/// at that point they *are* some typed program's own encoding, which is
/// what the field says. Anything else is reported as deliberately
/// malformed, which is the honest answer for bytes the typed language
/// does not express.
fn script_source_of(
    target: &ReviewedElementsTapscriptDefinition,
    script: &[u8],
) -> FixtureScriptSource {
    match TapscriptProgram::decode(target, script) {
        Ok(program) if program.encode(target) == script => FixtureScriptSource::TypedProgram,
        _ => FixtureScriptSource::DeliberatelyMalformed,
    }
}

/// One fixture's complete subject, plus the run-level facts it is stated
/// against.
fn projection(
    target: &ReviewedElementsTapscriptDefinition,
    fixture: &CompoundPrototypeFixture,
    binding: &ReviewedDevelopmentBinding,
    domain: WireExecutionDomain,
) -> PrototypeFixtureProjection {
    // The coherence check above already established that the executing
    // leaf is a leaf at the reviewed version, so the byte is read from
    // the fixture's own construction rather than from the contract: a
    // row must state what it stated, not what it should have.
    let leaf_version = fixture
        .reviewed_leaf_version()
        .map_or(0, target_elements::LeafVersion::get);
    PrototypeFixtureProjection {
        case: fixture.case.clone(),
        target_contract_version: fixture.target_contract_version,
        network_id: binding.binding().network_id(),
        genesis_id: binding.binding().genesis_id(),
        execution_domain: domain,
        leaf_version,
        leaf_version_status: LeafVersionStatus::Reviewed,
        enforcement_layer: EnforcementLayer::Consensus,
        script_source: script_source_of(target, &fixture.script),
        script: fixture.script.clone(),
        initial_stack: fixture.initial_stack.clone(),
        construction: fixture.construction.clone(),
        expected: fixture.expected,
        expected_resources: fixture.expected_resources,
        claims: fixture.claims.clone(),
    }
}

/// Whether the observation is what the fixture requires.
///
/// The verdict is compared always. The failure class is not: a compound
/// fixture states no admitted class set, so there is nothing to compare
/// it against, and inventing one here would fail an honest executor over
/// a distinction the fixture never drew. The exact resource rows are
/// compared under the same rule the primitive report uses, because a
/// disagreeing exact row means the executor ran something other than
/// what it was handed.
fn compare(fixture: &CompoundPrototypeFixture, observed: &ObservedNativeOutcome) -> CaseStatus {
    if observed.verdict == NativeVerdict::InfrastructureError {
        return CaseStatus::InfrastructureError;
    }
    let verdict_matched = match fixture.expected {
        ExpectedPrototypeOutcome::Accepted => observed.verdict == NativeVerdict::Accepted,
        ExpectedPrototypeOutcome::Rejected => observed.verdict == NativeVerdict::Rejected,
    };
    if verdict_matched
        && crate::validate::resources_agree(fixture.expected_resources, &observed.resources)
    {
        CaseStatus::Passed
    } else {
        CaseStatus::Failed
    }
}

/// The typed claim rows for one relation, in claim order.
///
/// Every claim the relation owns appears, including one no case bears
/// on: a claim missing from the array would be a corner of the relation
/// a reader could not see was unestablished.
fn claim_rows(
    relation: PrototypeRelation,
    per_claim: &BTreeMap<PrototypeClaim, Vec<(PrototypeCaseId, CaseStatus)>>,
) -> Vec<PrototypeClaimResult> {
    PrototypeClaim::ALL
        .iter()
        .filter(|claim| claim.relation() == relation)
        .map(|claim| {
            let bearing = per_claim.get(claim).map_or(&[][..], Vec::as_slice);
            let required = matches!(claim.requirement(), ClaimRequirement::Required);
            let unresolved_reason = match claim.requirement() {
                ClaimRequirement::Required => None,
                ClaimRequirement::Unresolved(reason) => Some(reason.to_owned()),
            };
            let statuses: Vec<CaseStatus> = bearing.iter().map(|(_, status)| *status).collect();

            let disposition = if statuses.contains(&CaseStatus::InfrastructureError) {
                EvidenceDisposition::InfrastructureError
            } else if statuses.contains(&CaseStatus::Failed) {
                EvidenceDisposition::Failed
            } else if statuses.is_empty() {
                if required {
                    EvidenceDisposition::Failed
                } else {
                    EvidenceDisposition::UnresolvedByDesign
                }
            } else {
                EvidenceDisposition::Passed
            };

            PrototypeClaimResult {
                claim: *claim,
                relation,
                required,
                unresolved_reason,
                bearing_cases: bearing.iter().map(|(case, _)| case.clone()).collect(),
                disposition,
            }
        })
        .collect()
}

/// The counts, and what they add up to.
fn summarize(
    relation: PrototypeRelation,
    cases: &[PrototypeCaseResult],
    claims: &[PrototypeClaimResult],
) -> PrototypeReportSummary {
    let count = |wanted: CaseStatus| {
        u32::try_from(cases.iter().filter(|case| case.status == wanted).count()).unwrap_or(u32::MAX)
    };
    let required: Vec<&PrototypeClaimResult> = claims.iter().filter(|row| row.required).collect();
    let required_passed = required
        .iter()
        .filter(|row| row.disposition == EvidenceDisposition::Passed)
        .count();
    let unresolved = claims
        .iter()
        .filter(|row| row.disposition == EvidenceDisposition::UnresolvedByDesign)
        .count();

    let cases_failed = count(CaseStatus::Failed);
    let cases_infrastructure_error = count(CaseStatus::InfrastructureError);
    let completeness = if required_passed != required.len()
        || cases_failed > 0
        || cases_infrastructure_error > 0
        || cases.is_empty()
    {
        // An empty matrix is a failure rather than a vacuous pass. A run
        // that executed nothing established nothing, and the one status
        // that must never be reachable without cases is the one that
        // reads as a complete prototype.
        PrototypeReportCompleteness::Failed
    } else if unresolved > 0 {
        PrototypeReportCompleteness::PartialUnresolvedClaims
    } else {
        match relation {
            PrototypeRelation::MetadataConstructorContinuity => {
                PrototypeReportCompleteness::CompleteForConstructorPrototype
            }
            PrototypeRelation::WideFloorRelation => {
                PrototypeReportCompleteness::CompleteForWideFloorPrototype
            }
        }
    };

    PrototypeReportSummary {
        cases_total: u32::try_from(cases.len()).unwrap_or(u32::MAX),
        cases_passed: count(CaseStatus::Passed),
        cases_failed,
        cases_infrastructure_error,
        claims_total: u32::try_from(claims.len()).unwrap_or(u32::MAX),
        required_claims_total: u32::try_from(required.len()).unwrap_or(u32::MAX),
        required_claims_passed: u32::try_from(required_passed).unwrap_or(u32::MAX),
        claims_unresolved: u32::try_from(unresolved).unwrap_or(u32::MAX),
        completeness,
    }
}

/// The report of an ad hoc prototype run, which is not evidence.
///
/// There is no route from here to [`prototype_gate`]: this type has no
/// validator, and the gate reads only a [`ValidatedPrototypeReport`]
///.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperimentalPrototypeReport {
    report: PrototypeConformanceReport,
}

impl ExperimentalPrototypeReport {
    /// The report.
    #[must_use]
    pub const fn report(&self) -> &PrototypeConformanceReport {
        &self.report
    }

    /// Consumes the wrapper, yielding the raw report.
    #[must_use]
    pub fn into_report(self) -> PrototypeConformanceReport {
        self.report
    }
}

/// Everything the prototype-report validator needs to recompute a report.
///
/// The relation is no longer among them: it travels with the matrix, so a
/// caller cannot state one relation and supply the other's rows.
#[derive(Clone, Copy, Debug)]
pub struct PrototypeReportValidationInputs<'a> {
    /// The exact reviewed contract.
    pub target: &'a ReviewedElementsTapscriptDefinition,
    /// The binding welded to that contract.
    pub binding: &'a ReviewedDevelopmentBinding,
    /// The canonical matrix that was executed, in canonical order.
    pub matrix: CanonicalPrototypeMatrix<'a>,
    /// What the executor answered.
    pub transcript: &'a ExecutionTranscript,
}

/// A prototype report every field of which has been recomputed and found
/// equal.
///
/// The wrapper has no public constructor other than
/// [`validate_prototype_report`]. That is the whole point: a raw report
/// is a description someone produced, and the gate must not accept a
/// description of a run in place of the run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedPrototypeReport {
    report: PrototypeConformanceReport,
}

impl ValidatedPrototypeReport {
    /// The validated report.
    #[must_use]
    pub const fn report(&self) -> &PrototypeConformanceReport {
        &self.report
    }

    /// Consumes the validated state, yielding the raw report.
    #[must_use]
    pub fn into_report(self) -> PrototypeConformanceReport {
        self.report
    }
}

/// Recomputes every field of an offered prototype report from its own
/// inputs.
///
/// # What is recomputed rather than trusted
///
/// The case census, each row's complete fixture projection, each row's
/// observation and recomputed status, the claim rows and their
/// dispositions, the summary counts, the completeness, the role and its
/// agreement with the relation, the observed environment, and the
/// executor provenance. Every comparison rejects in both directions: a
/// row the report omits and a row the report invents are both failures.
///
/// # Errors
///
/// [`NativeConformanceError::UnsupportedReportSchema`] for a revision
/// this harness does not validate, and then the first typed mismatch: a
/// case census that is not the matrix, a duplicated case, a projection
/// that is not the executed fixture, an observation that is not the
/// transcript's, claim rows that are not the recomputed ones, a role that
/// is not the relation's, or a summary that is not what the rows add up
/// to.
pub fn validate_prototype_report(
    report: PrototypeConformanceReport,
    inputs: PrototypeReportValidationInputs<'_>,
) -> Result<ValidatedPrototypeReport, NativeConformanceError> {
    if report.schema != PROTOTYPE_REPORT_SCHEMA {
        return Err(NativeConformanceError::UnsupportedReportSchema {
            offered: report.schema,
        });
    }
    // The role and the relation are two classifications of one run, and
    // a report whose two disagree has mislabelled one of them. Checked
    // on the offered report before the recomputation, so the answer names
    // the contradiction rather than a downstream census difference.
    if report.role != report.relation.report_role() {
        return Err(NativeConformanceError::PrototypeReportRoleMismatch);
    }
    // A revision-2 prototype report is not a revision-3 one, and is
    // refused by name rather than by a downstream field difference
    // (´[PLAN-rule:guide11:request-subject]´).
    if report.expectation_boundary != RequestExpectationBoundary::ExecutorReceivesSubjectOnly {
        return Err(
            NativeConformanceError::UnsupportedRequestExpectationBoundary {
                offered: report.expectation_boundary,
            },
        );
    }
    // The environment, at the validator's own boundary as well as inside
    // the recomputation (´[PLAN-rule:guide11:environment-twice]´).
    crate::executor::compare_environment(
        inputs.target,
        inputs.binding,
        inputs.transcript.environment(),
    )?;

    let recomputed = evaluate_prototypes(
        inputs.target,
        inputs.binding,
        inputs.matrix,
        inputs.transcript,
    )?;

    // The case census, duplicate-sensitively and in both directions.
    let mut seen: BTreeSet<PrototypeCaseId> = BTreeSet::new();
    for row in &report.cases {
        if !seen.insert(row.case()) {
            return Err(NativeConformanceError::DuplicatePrototypeCase(row.case()));
        }
    }
    let expected_cases: BTreeSet<PrototypeCaseId> = recomputed
        .cases
        .iter()
        .map(PrototypeCaseResult::case)
        .collect();
    if seen != expected_cases || report.cases.len() != recomputed.cases.len() {
        return Err(NativeConformanceError::PrototypeReportCaseCensusMismatch);
    }

    // Each row's complete subject and its recomputed outcome. The order
    // is the matrix's own, so a permutation is a census mismatch rather
    // than a reordering to be tolerated.
    for (offered, expected) in report.cases.iter().zip(&recomputed.cases) {
        if offered.case() != expected.case() {
            return Err(NativeConformanceError::PrototypeReportCaseCensusMismatch);
        }
        if offered.fixture != expected.fixture || offered.claims != expected.claims {
            return Err(NativeConformanceError::PrototypeProjectionMismatch(
                expected.case(),
            ));
        }
        if offered.observed != expected.observed || offered.status != expected.status {
            return Err(NativeConformanceError::PrototypeCaseOutcomeMismatch(
                expected.case(),
            ));
        }
    }

    // The typed claims, duplicate-sensitively and in both directions.
    let mut offered_claims: BTreeSet<PrototypeClaim> = BTreeSet::new();
    for row in &report.claims {
        if !offered_claims.insert(row.claim) {
            return Err(NativeConformanceError::DuplicatePrototypeClaim(row.claim));
        }
    }
    let expected_claims: BTreeSet<PrototypeClaim> =
        recomputed.claims.iter().map(|row| row.claim).collect();
    if let Some(missing) = expected_claims.difference(&offered_claims).next() {
        return Err(NativeConformanceError::MissingPrototypeClaim(*missing));
    }
    if let Some(extra) = offered_claims.difference(&expected_claims).next() {
        return Err(NativeConformanceError::UnexpectedPrototypeClaim(*extra));
    }
    if report.claims != recomputed.claims {
        return Err(NativeConformanceError::PrototypeReportClaimCensusMismatch);
    }

    if report.summary != recomputed.summary {
        return Err(NativeConformanceError::PrototypeReportSummaryMismatch);
    }
    if report.executor != recomputed.executor
        || report.observed_environment != recomputed.observed_environment
        || report.activation != recomputed.activation
    {
        return Err(NativeConformanceError::ReportProvenanceMismatch);
    }

    // Anything left is a field neither the matrix nor the run supplies:
    // the schema, the role, the relation, the boundary, the contract
    // revision, and the bound identifiers. A whole-value comparison
    // catches all of them at once and needs no per-field enumeration to
    // stay complete.
    if report != recomputed {
        return Err(NativeConformanceError::PrototypeReportSummaryMismatch);
    }

    Ok(ValidatedPrototypeReport { report })
}

/// Decides whether one prototype run is target-native evidence.
///
/// Accepts only a validated report: a raw report is a description of a
/// run, and the gate's question is about the run.
///
/// # Errors
///
/// [`NativeConformanceError::MockExecutorCannotSatisfyNativeGate`] for a
/// declared mock run, checked before anything else;
/// [`NativeConformanceError::EmptyPrototypeMatrix`] for a run with no
/// cases at all; then
/// [`NativeConformanceError::RequiredPrototypeClaimMissing`] or
/// [`NativeConformanceError::RequiredPrototypeClaimFailed`] for the first
/// required claim without passing case evidence; and then
/// [`NativeConformanceError::PrototypeCaseFailed`] or
/// [`NativeConformanceError::PrototypeCaseInfrastructureError`] for the
/// first case that did not pass.
pub fn prototype_gate(validated: &ValidatedPrototypeReport) -> Result<(), NativeConformanceError> {
    let report = &validated.report;
    if report.executor.declaration == ExecutorDeclaration::Mock {
        return Err(NativeConformanceError::MockExecutorCannotSatisfyNativeGate);
    }
    // A run of nothing is not a passing run. Without this the gate would
    // accept an empty matrix, whose every required claim would be
    // vacuously absent rather than failed only because there were no
    // claims either.
    if report.cases.is_empty() {
        return Err(NativeConformanceError::EmptyPrototypeMatrix);
    }

    // Claims before rows. A row's status already accounts for the claims
    // it bears on, but naming the claim is what tells a reader which
    // corner of the relation the run failed to establish.
    for row in &report.claims {
        if !row.required {
            continue;
        }
        match row.disposition {
            EvidenceDisposition::Passed => {}
            EvidenceDisposition::UnresolvedByDesign => {
                return Err(NativeConformanceError::RequiredPrototypeClaimMissing(
                    row.claim,
                ));
            }
            EvidenceDisposition::Failed | EvidenceDisposition::InfrastructureError => {
                return Err(if row.bearing_cases.is_empty() {
                    NativeConformanceError::RequiredPrototypeClaimMissing(row.claim)
                } else {
                    NativeConformanceError::RequiredPrototypeClaimFailed(row.claim)
                });
            }
        }
    }

    for row in &report.cases {
        match row.status {
            CaseStatus::Passed => {}
            CaseStatus::Failed => {
                return Err(NativeConformanceError::PrototypeCaseFailed(row.case()));
            }
            CaseStatus::InfrastructureError => {
                return Err(NativeConformanceError::PrototypeCaseInfrastructureError(
                    row.case(),
                ));
            }
        }
    }

    Ok(())
}
