//! The Guide-9 evidence plan, the comparison, and the gate.
//!
//! # The plan is first-party policy
//!
//! Which evidence requirements Guide 9 must establish, which it
//! deliberately leaves unresolved, which the reviewed static contract
//! marks unsupported, and which wait for complete transaction evidence
//! is a decision this repository makes. No executor supplies it, and no
//! executor can widen or narrow it: an executor lacking a feature does
//! not remove a required row, it fails one.
//!
//! # The comparison compares
//!
//! The fixture's stated expectation and the executor's reported
//! observation are two independent values, and a case passes only when
//! they agree exactly — verdict, stacks, and failure class. A mismatch
//! is a finding to triage against the four possible owners (contract
//! transcription, executor, target behaviour, fixture), never a reason
//! to adopt the observation as the new expectation.
//!
//! # A broad row is not established by one case
//!
//! Aggregating cases straight onto a broad evidence requirement is
//! existential, and one passing case would move a whole dimension to
//! passed. Beneath every row sits the typed claim census
//! ([`crate::claim`]): a row passes only when every claim it requires has
//! a passing case bearing on it, and a claim no passing case bears on is
//! recorded as unresolved with its reason rather than absorbed into a
//! passing row.
//!
//! # The gate is not the report
//!
//! A report describes a run. Describing a run is not the run having
//! happened that way, so the gate does not read a report: it reads a
//! [`ValidatedNativeConformanceReport`], which exists only once every
//! field of the raw report has been recomputed from the fixtures, the
//! plan, the claim registry, and the transcript, and found equal in both
//! directions. Removing a failed row, relabelling it unresolved, clearing
//! the evidence array, duplicating a passing row, or editing the summary
//! all fail there rather than passing here.
//!
//! The gate then refuses a declared mock run before anything else: a
//! mock's answers come from the census's own expectations, obtained out
//! of band since revision 3 stopped sending them, so a green mock report
//! says only that the harness can compare a value with itself.

use std::collections::{BTreeMap, BTreeSet};

use target_elements::{
    ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition, TargetEvidenceRequirementId,
};

use crate::claim::{ClaimRegistry, NativeEvidenceClaim};
use crate::error::NativeConformanceError;
use crate::executor::{ExecutionTranscript, ExecutorTrust};
use crate::fixture::{
    CanonicalPrimitiveFixtureSet, EnforcementLayer, ExpectedPrimitiveOutcome,
    ExpectedResourceObservation, LeafVersionStatus, NativeCaseGroup, NativeCaseId,
    PrimitiveFixture, PrimitiveFixtureSet, ResourceExpectation, canonical_fixture_set,
};
use crate::protocol::{
    NativeResourceObservation, NativeVerdict, RequestExpectationBoundary, WireEnvironment,
    WireExecutionDomain,
};
use crate::provenance::{ExpectedExecutorProvenance, validate_executor_provenance};
use crate::report::{
    ActivationRecord, CaseStatus, EvidenceDisposition, EvidencePlanClass,
    EvidenceRequirementResult, ExecutorDeclaration, ExecutorProvenance, NATIVE_REPORT_SCHEMA,
    NativeCaseResult, NativeConformanceReport, NativeEvidenceClaimResult, NativeReportSummary,
    ObservedEnvironment, ObservedNativeOutcome, PrototypeReportRole, ReportCompleteness,
};
use crate::vocabulary::{capability_name, evidence_requirement_name};

/// Where the Guide-9 plan puts each evidence requirement.
///
/// The rows outside the required set are stated with their reason:
///
/// - sighash semantics remain unresolved because no sighash dimension
///   is promoted to reviewed in the static contract, and a signature
///   primitive existing is not sighash evidence;
/// - commitment equality is unsupported because the reviewed contract
///   describes no target mechanism for it, and low-level curve and hash
///   primitives are not one;
/// - confidential-value conservation is deferred because it is a
///   whole-transaction property, which needs complete transaction
///   evidence rather than a script-level case;
/// - policy resource limits are unresolved because every resource case
///   this census states is stated at the consensus layer, and a
///   consensus acceptance is not evidence about what a node's relay
///   rules decline to forward. The row returns to the required set when
///   an actual relay-policy matrix exists to establish it.
const EVIDENCE_PLAN: &[(TargetEvidenceRequirementId, EvidencePlanClass)] = &[
    (
        TargetEvidenceRequirementId::TapscriptExecutionDomain,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::LeafVersionActivation,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::OpcodeSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::EncodingSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::PushEncodingSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::InputIntrospectionSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::OutputIntrospectionSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::TransactionIntrospectionSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::ArithmeticSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::ComparisonSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::ConversionSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::StreamingHashSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::SignatureSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::RelativeTimelockSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::EllipticCurveSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::IssuanceIntrospection,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::ConsensusResourceLimits,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::PolicyResourceLimits,
        EvidencePlanClass::UnresolvedByDesign,
    ),
    // The compound-proof substrate. Every one of these is exercised by
    // static cases with no transaction context, so all three are
    // required rather than deferred.
    (
        TargetEvidenceRequirementId::StackRearrangementSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::ByteStringSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::VerificationSemantics,
        EvidencePlanClass::Required,
    ),
    (
        TargetEvidenceRequirementId::SighashSemantics,
        EvidencePlanClass::UnresolvedByDesign,
    ),
    (
        TargetEvidenceRequirementId::CommitmentEquality,
        EvidencePlanClass::UnsupportedByStaticContract,
    ),
    (
        TargetEvidenceRequirementId::ConfidentialValueConservation,
        EvidencePlanClass::DeferredToTransactionEvidence,
    ),
];

/// The Guide-9 partition of the target evidence census.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidencePlan {
    classes: BTreeMap<TargetEvidenceRequirementId, EvidencePlanClass>,
}

impl EvidencePlan {
    /// Where the plan puts one requirement.
    #[must_use]
    pub fn class(&self, id: TargetEvidenceRequirementId) -> Option<EvidencePlanClass> {
        self.classes.get(&id).copied()
    }

    /// Every requirement, with its class, in requirement order.
    pub fn iter(&self) -> impl Iterator<Item = (TargetEvidenceRequirementId, EvidencePlanClass)> {
        self.classes.iter().map(|(id, class)| (*id, *class))
    }
}

/// The Guide-9 evidence plan.
///
/// # Errors
///
/// [`NativeConformanceError::EvidenceCensusMismatch`] when the plan does
/// not name exactly the target's evidence census: a requirement admitted
/// upstream without a plan class, a class stated twice, or a class
/// stated for a requirement the census does not hold.
pub fn guide_nine_evidence_plan() -> Result<EvidencePlan, NativeConformanceError> {
    let mut classes = BTreeMap::new();
    for (id, class) in EVIDENCE_PLAN {
        if classes.insert(*id, *class).is_some() {
            return Err(NativeConformanceError::EvidenceCensusMismatch);
        }
    }
    if classes.len() != TargetEvidenceRequirementId::ALL.len()
        || !TargetEvidenceRequirementId::ALL
            .iter()
            .all(|id| classes.contains_key(id))
    {
        return Err(NativeConformanceError::EvidenceCensusMismatch);
    }
    Ok(EvidencePlan { classes })
}

/// Why one required evidence row states no required claim of its own.
///
/// A typed, documented exception rather than a tolerated absence. The
/// invariant below exists because a broad required row whose defining
/// claims are all unresolved can pass on case aggregation alone — the
/// exact shape that let consensus cases carry the policy resource row —
/// and an exception to it must therefore say, in the source, which row
/// is exempt and why claim-level decomposition does not apply to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimDecompositionException {
    /// The required row that owns no required claim.
    pub requirement: TargetEvidenceRequirementId,
    /// Why decomposing it into claims does not apply.
    pub reason: &'static str,
}

/// Every required row exempt from claim-level decomposition.
///
/// Four rows, each stated with its reason, and the reasons are of two
/// kinds. Every other row the plan requires is defined by at least one
/// required claim of its own, which is what makes that row's disposition
/// a statement about the dimension rather than about whichever cases
/// happened to be filed under it.
///
/// # What an exception is not
///
/// It is not permission for the row to pass without evidence. Every row
/// here is still borne on by cases, still fails when one of them fails,
/// and — since Wave 3 — sits behind a gate that refuses any failed case
/// whatever it bears on. What the exception records is that the
/// claim-level statement about this dimension is written down somewhere
/// else, named here, rather than being absent.
const CLAIM_DECOMPOSITION_EXCEPTIONS: &[ClaimDecompositionException] = &[
    ClaimDecompositionException {
        requirement: TargetEvidenceRequirementId::EncodingSemantics,
        reason: "field encoding is decomposed into claims, but under the rows that own the \
                 cases: the only cases bearing on this row are the input and output \
                 introspection cases, and their required explicit-form claims are exactly \
                 the statement that a field decodes to one exact stack form. A claim \
                 restating that here would be the same observation counted twice, and two \
                 copies of one observation can disagree",
    },
    ClaimDecompositionException {
        requirement: TargetEvidenceRequirementId::StackRearrangementSemantics,
        reason: "the row's whole content is that each reviewed rearrangement primitive moves \
                 the stack exactly as the contract states, which the per-case comparison \
                 establishes case by case against a stated final stack. Its cases name \
                 primitives, so they carry the required primitive success and abort claims \
                 under the opcode row; a further claim would restate the comparison rather \
                 than decompose the dimension",
    },
    ClaimDecompositionException {
        requirement: TargetEvidenceRequirementId::ByteStringSemantics,
        reason: "as for stack rearrangement: the dimension is the exact byte-string result of \
                 each reviewed primitive, established by comparison against a stated final \
                 stack, and its cases carry the required primitive claims under the opcode \
                 row",
    },
    ClaimDecompositionException {
        requirement: TargetEvidenceRequirementId::VerificationSemantics,
        reason: "as for stack rearrangement: the dimension is that each reviewed verification \
                 primitive continues or aborts exactly where the contract says, established \
                 by comparison of verdict and failure class, and its cases carry the required \
                 primitive success and abort claims under the opcode row",
    },
];

/// The exceptions, for the crate's own tests.
#[cfg(test)]
pub(crate) const fn claim_decomposition_exceptions_for_tests()
-> &'static [ClaimDecompositionException] {
    CLAIM_DECOMPOSITION_EXCEPTIONS
}

/// Whether every required row is defined by at least one required claim.
///
/// # What this stops
///
/// A row classified `Required` whose owned claims are all unresolved
/// passes as soon as any case is filed under it, because
/// `missing_required_claims` is then vacuously empty. The row then reads
/// as an established dimension while the claim that defines it records
/// that nothing established it — an internally contradictory report the
/// gate cannot notice, since it reads the row.
///
/// Checked at every report construction rather than only at startup: the
/// plan and the registry are two tables that can drift apart in one
/// edit, and the report is where the drift would become a claim.
///
/// # Errors
///
/// [`NativeConformanceError::RequiredRowWithoutRequiredClaim`] for the
/// first required row that owns no required claim and has no stated
/// exception.
pub fn check_required_rows_own_required_claims(
    plan: &EvidencePlan,
    registry: &ClaimRegistry,
) -> Result<(), NativeConformanceError> {
    for (id, class) in plan.iter() {
        if class != EvidencePlanClass::Required {
            continue;
        }
        if !registry.required_claims(id).is_empty() {
            continue;
        }
        if CLAIM_DECOMPOSITION_EXCEPTIONS
            .iter()
            .any(|exception| exception.requirement == id && !exception.reason.trim().is_empty())
        {
            continue;
        }
        return Err(NativeConformanceError::RequiredRowWithoutRequiredClaim(id));
    }
    Ok(())
}

/// Which evidence requirements one fixture group bears on.
///
/// A case exercising a primitive also bears on opcode semantics, which
/// is added separately: the group says what the case is about, the
/// primitive says that a reviewed byte executed at all.
///
/// # What was corrected here
///
/// Instruction encoding used to bear on *field* encoding semantics as
/// well. It does not: a case establishing that one byte decodes as one
/// primitive says nothing about the prefix an explicit asset carries.
/// Field encodings are established by the introspection cases, which are
/// the cases that read a field and state its exact stack form — and
/// which is also what the reviewed contract says, since those are the
/// primitives whose declared evidence includes the encoding
/// requirement.
///
/// Resource limits stay split between their two rows deliberately, and
/// the enforcement layer is what splits them. The literal-width boundary
/// is the target's own rule and bears on the consensus row; the
/// nonminimal forms are valid spends that nodes decline to forward,
/// which is the relay row and nothing else. A group alone cannot say
/// which of the two a case is about, which is why the layer is a
/// parameter here rather than a comment: crediting a consensus
/// acceptance to the relay row would report a policy observation the run
/// never made.
const fn requirements_of(
    group: NativeCaseGroup,
    layer: EnforcementLayer,
) -> &'static [TargetEvidenceRequirementId] {
    match group {
        NativeCaseGroup::ExecutionDomain => {
            &[TargetEvidenceRequirementId::TapscriptExecutionDomain]
        }
        NativeCaseGroup::LeafVersion => &[TargetEvidenceRequirementId::LeafVersionActivation],
        NativeCaseGroup::InstructionEncoding => &[TargetEvidenceRequirementId::OpcodeSemantics],
        NativeCaseGroup::PushEncoding => &[TargetEvidenceRequirementId::PushEncodingSemantics],
        NativeCaseGroup::InputIntrospection => &[
            TargetEvidenceRequirementId::InputIntrospectionSemantics,
            TargetEvidenceRequirementId::EncodingSemantics,
        ],
        NativeCaseGroup::OutputIntrospection => &[
            TargetEvidenceRequirementId::OutputIntrospectionSemantics,
            TargetEvidenceRequirementId::EncodingSemantics,
        ],
        NativeCaseGroup::TransactionIntrospection => {
            &[TargetEvidenceRequirementId::TransactionIntrospectionSemantics]
        }
        NativeCaseGroup::StackRearrangement => {
            &[TargetEvidenceRequirementId::StackRearrangementSemantics]
        }
        NativeCaseGroup::ByteString => &[TargetEvidenceRequirementId::ByteStringSemantics],
        NativeCaseGroup::Verification => &[TargetEvidenceRequirementId::VerificationSemantics],
        NativeCaseGroup::Arithmetic => &[TargetEvidenceRequirementId::ArithmeticSemantics],
        NativeCaseGroup::Comparison => &[TargetEvidenceRequirementId::ComparisonSemantics],
        NativeCaseGroup::Conversion => &[TargetEvidenceRequirementId::ConversionSemantics],
        NativeCaseGroup::StreamingHash => &[TargetEvidenceRequirementId::StreamingHashSemantics],
        NativeCaseGroup::EllipticCurve => &[TargetEvidenceRequirementId::EllipticCurveSemantics],
        NativeCaseGroup::Signature => &[TargetEvidenceRequirementId::SignatureSemantics],
        NativeCaseGroup::Sighash => &[TargetEvidenceRequirementId::SighashSemantics],
        NativeCaseGroup::RelativeTimelock => {
            &[TargetEvidenceRequirementId::RelativeTimelockSemantics]
        }
        NativeCaseGroup::ConfidentialValue => {
            &[TargetEvidenceRequirementId::ConfidentialValueConservation]
        }
        NativeCaseGroup::Issuance => &[
            TargetEvidenceRequirementId::IssuanceIntrospection,
            TargetEvidenceRequirementId::InputIntrospectionSemantics,
        ],
        NativeCaseGroup::Resource => match layer {
            EnforcementLayer::Consensus => &[TargetEvidenceRequirementId::ConsensusResourceLimits],
            EnforcementLayer::RelayPolicy => &[TargetEvidenceRequirementId::PolicyResourceLimits],
        },
    }
}

/// The requirements one group and layer bear on, for the crate's own
/// tests.
#[cfg(test)]
pub(crate) const fn requirements_for_tests(
    group: NativeCaseGroup,
    layer: EnforcementLayer,
) -> &'static [TargetEvidenceRequirementId] {
    requirements_of(group, layer)
}

/// Builds the report of one canonical run.
///
/// This is the native evidence path. It accepts only a
/// [`CanonicalPrimitiveFixtureSet`], and it does not take that wrapper's
/// word for it: the census is regenerated from the reviewed contract and
/// the binding, and every fixture's complete projection is compared
/// against the regenerated one before a single claim is derived
///.
///
/// # Why the subject is checked rather than the script
///
/// A caller-chosen census can be made to look like evidence without any
/// executor misbehaving: the claims below read a case's declared group,
/// primitive, and stated outcome, so a program that merely pushes a true
/// literal, filed under a case naming the signature primitive, is
/// credited with an accepted transaction signature. Checking that the
/// named primitive *appears* in the script would not close that either —
/// a script may contain an opcode it never reaches, or reach it in a
/// context unrelated to the claimed property. Evidence-bearing membership
/// is therefore defined by the canonical census and by nothing else.
///
/// # Errors
///
/// [`NativeConformanceError::NoncanonicalFixtureCensus`] or
/// [`NativeConformanceError::NoncanonicalFixtureSubject`] when the
/// offered census is not the regenerated canonical one,
/// [`NativeConformanceError::TargetContractMismatch`] or
/// [`NativeConformanceError::DevelopmentBindingMismatch`] when a fixture
/// was stated against a different contract or network from the run's,
/// and [`NativeConformanceError::MissingCaseResponse`] when the
/// transcript does not answer a fixture.
pub fn evaluate(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    fixtures: &CanonicalPrimitiveFixtureSet,
    transcript: &ExecutionTranscript,
    plan: &EvidencePlan,
    registry: &ClaimRegistry,
) -> Result<NativeConformanceReport, NativeConformanceError> {
    let regenerated = canonical_fixture_set(target, binding)?;
    if regenerated.len() != fixtures.len() {
        return Err(NativeConformanceError::NoncanonicalFixtureCensus);
    }
    // Pairwise in canonical case order, so a census holding the right
    // cases in a different order is caught as well. Declaration order is
    // not compared and cannot be: the census is a map keyed by case
    // identity, so permuting the declarations produces one value.
    for (offered, canonical) in fixtures.iter().zip(regenerated.iter()) {
        if offered.case() != canonical.case() {
            return Err(NativeConformanceError::NoncanonicalFixtureCensus);
        }
        // The complete projection, which is every member the invariant
        // names: program, stack, context, expected outcome, enforcement
        // layer, leaf version, script provenance, resources, and the
        // claim set the case owns.
        if offered.projection() != canonical.projection() {
            return Err(NativeConformanceError::NoncanonicalFixtureSubject(
                canonical.case(),
            ));
        }
    }

    evaluate_census(
        target,
        binding,
        fixtures.fixtures(),
        transcript,
        plan,
        registry,
        PrototypeReportRole::PrimitiveConformance,
    )
}

/// Builds the report of one ad hoc run.
///
/// An arbitrary census may still be executed and described — that is what
/// makes the fixture language useful for experiments and for this
/// harness's own protocol tests. What it may not do is become evidence:
/// the result is an [`ExperimentalPrimitiveReport`], which no validator
/// and no gate accepts, and whose recorded role says so in the serialized
/// document as well as in the type.
///
/// # Errors
///
/// The errors [`evaluate`] states, other than the two canonical-subject
/// ones, which cannot arise here.
pub fn evaluate_experimental(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    fixtures: &PrimitiveFixtureSet,
    transcript: &ExecutionTranscript,
    plan: &EvidencePlan,
    registry: &ClaimRegistry,
) -> Result<ExperimentalPrimitiveReport, NativeConformanceError> {
    Ok(ExperimentalPrimitiveReport {
        report: evaluate_census(
            target,
            binding,
            fixtures,
            transcript,
            plan,
            registry,
            PrototypeReportRole::ExperimentalPrimitive,
        )?,
    })
}

/// The report of one run over one census, under a stated role.
///
/// The comparison, the claim derivation, and the counting are one body
/// for both trust states deliberately: an experimental report that
/// described a run differently from the canonical one would be useless
/// for the experiments it exists to serve. What differs between the two
/// paths is which censuses may reach them and what the result can be used
/// for, and both of those are settled before this is called.
fn evaluate_census(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    fixtures: &PrimitiveFixtureSet,
    transcript: &ExecutionTranscript,
    plan: &EvidencePlan,
    registry: &ClaimRegistry,
    role: PrototypeReportRole,
) -> Result<NativeConformanceReport, NativeConformanceError> {
    // The plan and the registry must agree about what a required row
    // means before either is used to describe a run.
    check_required_rows_own_required_claims(plan, registry)?;

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

    transcript_run_binding(target, binding, transcript)?;
    // An answer with no question. A transcript whose two halves do not
    // correspond describes no run at all, and a report built from one
    // would present the responses of some other exchange.
    if let Some(case) = transcript
        .responses()
        .keys()
        .find(|case| !transcript.requests().contains_key(case))
    {
        return Err(NativeConformanceError::UnrequestedCaseResponse(*case));
    }

    let mut cases = Vec::new();
    let mut per_requirement: BTreeMap<TargetEvidenceRequirementId, Vec<CaseStatus>> =
        BTreeMap::new();
    let mut per_claim: BTreeMap<NativeEvidenceClaim, Vec<(NativeCaseId, CaseStatus)>> =
        BTreeMap::new();

    for fixture in fixtures {
        // The leaf version is checked against what the fixture says it
        // is: a reviewed case must be stated at the contract's leaf, and
        // an unreviewed one must not be, since a case claiming to
        // exercise an unreviewed leaf at the reviewed byte would
        // establish nothing.
        let leaf_agrees = match fixture.leaf_version_status() {
            LeafVersionStatus::Reviewed => {
                fixture.leaf_version() == definition.leaf_version().get()
            }
            LeafVersionStatus::Unreviewed => {
                fixture.leaf_version() != definition.leaf_version().get()
            }
        };
        if fixture.target_contract_version() != definition.version().get()
            || fixture.execution_domain() != domain
            || !leaf_agrees
        {
            return Err(NativeConformanceError::TargetContractMismatch);
        }
        if fixture.network_id() != binding.binding().network_id()
            || fixture.genesis_id() != binding.binding().genesis_id()
        {
            return Err(NativeConformanceError::DevelopmentBindingMismatch);
        }

        let case = fixture.case();
        // The subject being reported must be the subject that was sent.
        // The expectation is deliberately not compared and could not be:
        // under protocol revision 3 it never left this process, which is
        // what makes the comparison below a comparison of the *question*
        // rather than of the answer.
        let requested = transcript
            .requests()
            .get(&case)
            .ok_or(NativeConformanceError::MissingCaseRequest(case))?;
        if requested != &fixture.subject() {
            return Err(NativeConformanceError::TranscriptSubjectMismatch(case));
        }
        let response = transcript
            .responses()
            .get(&case)
            .ok_or(NativeConformanceError::MissingCaseResponse(case))?;

        let observed = ObservedNativeOutcome {
            verdict: response.verdict,
            final_stack: response.final_stack.clone(),
            final_altstack: response.final_altstack.clone(),
            observed_failure: response.observed_failure,
            resources: response.resources,
        };
        let status = compare(fixture, &observed);
        let projection = fixture.projection();

        for requirement in bearing_requirements(fixture) {
            per_requirement.entry(requirement).or_default().push(status);
        }
        for claim in &projection.claims {
            per_claim.entry(*claim).or_default().push((case, status));
        }

        cases.push(NativeCaseResult {
            claims: projection.claims.clone(),
            fixture: projection,
            observed,
            status,
        });
    }

    let claims = claim_rows(registry, plan, &per_claim);
    let evidence = evidence_rows(plan, registry, &per_requirement, &claims);
    let summary = summarize(&cases, &evidence, &claims);
    let observation = transcript.environment();

    Ok(NativeConformanceReport {
        schema: NATIVE_REPORT_SCHEMA,
        role,
        target_contract_version: definition.version().get(),
        expectation_boundary: RequestExpectationBoundary::ExecutorReceivesSubjectOnly,
        environment: WireEnvironment::Development,
        network_id: binding.binding().network_id(),
        genesis_id: binding.binding().genesis_id(),
        activation: activation_record_of(binding),
        observed_environment: ObservedEnvironment {
            environment: observation.environment,
            chain_name: observation.chain_name.clone(),
            network_id: observation.network_id,
            genesis_id: observation.genesis_id,
            active_domains: observation.active_domains.clone(),
            active_leaf_versions: observation.active_leaf_versions.clone(),
        },
        executor: provenance_of(transcript),
        cases,
        evidence,
        claims,
        summary,
    })
}

/// Whether this transcript is a run under this contract and this binding.
///
/// # The weld, stated once
///
/// A transcript retains the target projection and the deployment
/// projection the run was requested under, so the question is answered by
/// exact typed comparison rather than by the case identities happening to
/// line up `(´[PLAN-rule:guide11-exec:transcript-binding]´)`.
///
/// The environment is then compared a second time. The first comparison
/// happened at the handshake, under whichever binding the *run* was
/// requested with; this one happens under the binding the *report* is
/// being stated against, so a rebound transcript fails here even if some
/// unforeseen path reached a report with the two projections agreeing
/// `(´[PLAN-rule:guide11-exec:environment-twice]´)`.
///
/// Shared by the primitive and prototype paths, because a transcript is
/// bound the same way whichever workload produced it, and two copies of
/// this would eventually disagree.
///
/// # Errors
///
/// [`NativeConformanceError::TranscriptTargetRebinding`],
/// [`NativeConformanceError::TranscriptDeploymentRebinding`], and the
/// environment refusals [`crate::executor::compare_environment`] states.
pub(crate) fn transcript_run_binding(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ReviewedDevelopmentBinding,
    transcript: &ExecutionTranscript,
) -> Result<(), NativeConformanceError> {
    if transcript.target() != &target.projection() {
        return Err(NativeConformanceError::TranscriptTargetRebinding);
    }
    if transcript.deployment() != &binding.projection() {
        return Err(NativeConformanceError::TranscriptDeploymentRebinding);
    }
    crate::executor::compare_environment(target, binding, transcript.environment())
}

/// The report of an ad hoc run, which is not evidence.
///
/// # There is no route from here to the gate
///
/// This type has no validator, and [`gate`] does not accept it. That is
/// the whole of its meaning: the run happened, the report describes it
/// faithfully, and the subject was a census the caller chose rather than
/// the repository's evidence plan — so what the run establishes about the
/// target is whatever the reader makes of it, and not a claim this
/// harness certifies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperimentalPrimitiveReport {
    report: NativeConformanceReport,
}

impl ExperimentalPrimitiveReport {
    /// The report.
    #[must_use]
    pub const fn report(&self) -> &NativeConformanceReport {
        &self.report
    }

    /// Consumes the wrapper, yielding the raw report.
    #[must_use]
    pub fn into_report(self) -> NativeConformanceReport {
        self.report
    }
}

/// Everything the report validator needs to recompute a report.
#[derive(Clone, Copy, Debug)]
pub struct NativeReportValidationInputs<'a> {
    /// The exact reviewed contract.
    pub target: &'a ReviewedElementsTapscriptDefinition,
    /// The binding welded to that contract.
    pub binding: &'a ReviewedDevelopmentBinding,
    /// The canonical fixture census that was executed.
    pub fixtures: &'a CanonicalPrimitiveFixtureSet,
    /// The evidence plan.
    pub plan: &'a EvidencePlan,
    /// The typed claim census.
    pub registry: &'a ClaimRegistry,
    /// What the executor answered.
    pub transcript: &'a ExecutionTranscript,
}

/// A report every field of which has been recomputed and found equal.
///
/// The wrapper has no public constructor other than
/// [`validate_native_report`]. That is the whole point: a raw report is a
/// description someone produced, and the gate must not accept a
/// description of a run in place of the run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedNativeConformanceReport {
    report: NativeConformanceReport,
}

impl ValidatedNativeConformanceReport {
    /// The validated report.
    #[must_use]
    pub const fn report(&self) -> &NativeConformanceReport {
        &self.report
    }

    /// Consumes the validated state, yielding the raw report.
    #[must_use]
    pub fn into_report(self) -> NativeConformanceReport {
        self.report
    }

    /// Asserts the validated state over a raw report, for gate tests.
    ///
    /// Test-only, and crate-visible: no caller outside this crate can
    /// reach it, so it widens nothing. It exists because [`gate`] and
    /// [`validate_native_report`] are two separate rules, and a test of
    /// the first must be able to present a report the second would
    /// refuse — a run whose summary is failed, say, which the
    /// recomputation path can no longer produce over the canonical
    /// census because every canonical case bears on a required row.
    #[cfg(test)]
    pub(crate) const fn wrap_for_tests(report: NativeConformanceReport) -> Self {
        Self { report }
    }
}

/// Recomputes every field of an offered report from its own inputs.
///
/// # What is recomputed rather than trusted
///
/// The case census, each case's complete fixture projection, each case's
/// observation and recomputed status, the evidence rows and their plan
/// classes, the typed claim rows and their dispositions, the summary
/// counts, the completeness, the observed environment, and the executor
/// provenance. Every comparison rejects in both directions: a row the
/// report omits and a row the report invents are both failures.
///
/// # The subject is recomputed too
///
/// The recomputation runs through [`evaluate`], so validating a report
/// regenerates the canonical census and compares every fixture's complete
/// projection against it. A report about a subject that is not the
/// canonical evidence plan therefore fails here, before any question
/// about whether the report faithfully describes that subject — which it
/// may well do.
///
/// # Errors
///
/// [`NativeConformanceError::UnsupportedReportSchema`] for a report
/// revision this harness does not validate, and then the first typed
/// mismatch: a census that is not the canonical one, a case census that
/// is not the fixture census, a duplicated
/// case, a fixture projection that is not the executed fixture, an
/// observation that is not the transcript's, evidence or claim rows that
/// are not the recomputed ones, or a summary that is not what the rows
/// add up to.
pub fn validate_native_report(
    report: NativeConformanceReport,
    inputs: NativeReportValidationInputs<'_>,
) -> Result<ValidatedNativeConformanceReport, NativeConformanceError> {
    if report.schema != NATIVE_REPORT_SCHEMA {
        return Err(NativeConformanceError::UnsupportedReportSchema {
            offered: report.schema,
        });
    }
    // A revision-2 report is a document about a run whose executor was
    // handed the answer. It remains exactly that; what it is not is a
    // report this harness validates, and saying so by name here is the
    // difference between a loud refusal and a downstream field mismatch
    // that a reader would have to decode
    // (´[PLAN-rule:guide11-exec:request-subject]´).
    if report.expectation_boundary != RequestExpectationBoundary::ExecutorReceivesSubjectOnly {
        return Err(
            NativeConformanceError::UnsupportedRequestExpectationBoundary {
                offered: report.expectation_boundary,
            },
        );
    }
    // The environment, checked here as well as inside the recomputation.
    // The recomputation would reach it, but only by a path that must stay
    // reachable; this one is stated at the validator's own boundary so
    // that a rebound transcript fails whatever the path
    // (´[PLAN-rule:guide11-exec:environment-twice]´).
    crate::executor::compare_environment(
        inputs.target,
        inputs.binding,
        inputs.transcript.environment(),
    )?;

    let recomputed = evaluate(
        inputs.target,
        inputs.binding,
        inputs.fixtures,
        inputs.transcript,
        inputs.plan,
        inputs.registry,
    )?;

    // The case census, duplicate-sensitively and in both directions.
    let mut seen: BTreeSet<NativeCaseId> = BTreeSet::new();
    for row in &report.cases {
        if !seen.insert(row.case()) {
            return Err(NativeConformanceError::DuplicateReportCase(row.case()));
        }
    }
    let expected_cases: BTreeSet<NativeCaseId> = recomputed
        .cases
        .iter()
        .map(NativeCaseResult::case)
        .collect();
    if seen != expected_cases || report.cases.len() != recomputed.cases.len() {
        return Err(NativeConformanceError::ReportCaseCensusMismatch);
    }

    // Each row's complete subject and its recomputed outcome. The order
    // is canonical, so a permutation is a census mismatch rather than a
    // reordering to be tolerated.
    for (offered, expected) in report.cases.iter().zip(&recomputed.cases) {
        if offered.case() != expected.case() {
            return Err(NativeConformanceError::ReportCaseCensusMismatch);
        }
        if offered.fixture != expected.fixture || offered.claims != expected.claims {
            return Err(NativeConformanceError::FixtureProjectionMismatch(
                expected.case(),
            ));
        }
        if offered.observed != expected.observed || offered.status != expected.status {
            return Err(NativeConformanceError::ReportCaseOutcomeMismatch(
                expected.case(),
            ));
        }
    }

    // The typed claims, duplicate-sensitively and in both directions.
    let mut offered_claims: BTreeSet<NativeEvidenceClaim> = BTreeSet::new();
    for row in &report.claims {
        if !offered_claims.insert(row.claim) {
            return Err(NativeConformanceError::DuplicateEvidenceClaim(row.claim));
        }
    }
    let expected_claims: BTreeSet<NativeEvidenceClaim> =
        recomputed.claims.iter().map(|row| row.claim).collect();
    if let Some(missing) = expected_claims.difference(&offered_claims).next() {
        return Err(NativeConformanceError::MissingEvidenceClaim(*missing));
    }
    if let Some(extra) = offered_claims.difference(&expected_claims).next() {
        return Err(NativeConformanceError::UnexpectedEvidenceClaim(*extra));
    }
    if report.claims != recomputed.claims {
        return Err(NativeConformanceError::ReportClaimCensusMismatch);
    }

    if report.evidence != recomputed.evidence {
        return Err(NativeConformanceError::ReportEvidenceCensusMismatch);
    }
    if report.summary != recomputed.summary {
        return Err(NativeConformanceError::ReportSummaryMismatch);
    }
    if report.executor != recomputed.executor
        || report.observed_environment != recomputed.observed_environment
        || report.activation != recomputed.activation
    {
        return Err(NativeConformanceError::ReportProvenanceMismatch);
    }

    // Anything left is a field neither the census nor the run supplies:
    // the schema, the role, the boundary, the contract revision, and the
    // bound identifiers. A whole-value comparison catches all of them at
    // once and needs no per-field enumeration to stay complete.
    if report != recomputed {
        return Err(NativeConformanceError::ReportSummaryMismatch);
    }

    Ok(ValidatedNativeConformanceReport { report })
}

/// Decides whether one run is target-native evidence.
///
/// Accepts only a validated report: a raw report is a description of a
/// run, and the gate's question is about the run.
///
/// # Errors
///
/// [`NativeConformanceError::MockExecutorCannotSatisfyNativeGate`] for a
/// declared mock run, checked before anything else, then
/// [`NativeConformanceError::ExpectedProvenanceUnavailable`] when no
/// expectation was configured and
/// [`NativeConformanceError::ExecutorProvenanceUnestablished`] when the
/// run's provenance is not the expected one, then
/// [`NativeConformanceError::RequiredClaimMissing`] or
/// [`NativeConformanceError::RequiredClaimFailed`] for the first required
/// claim without passing case evidence, and then
/// [`NativeConformanceError::RequiredEvidenceMissing`],
/// [`NativeConformanceError::RequiredEvidenceFailed`], or
/// [`NativeConformanceError::RequiredEvidenceInfrastructureError`] for
/// the first required row that does not pass, then
/// [`NativeConformanceError::NativeCaseFailed`] or
/// [`NativeConformanceError::NativeCaseInfrastructureError`] for the
/// first case that did not pass whatever it bears on, and finally
/// [`NativeConformanceError::ReportSummaryFailed`] for a report whose
/// own summary records the run as failed.
pub fn gate(
    validated: &ValidatedNativeConformanceReport,
    expected_provenance: Option<&ExpectedExecutorProvenance>,
) -> Result<(), NativeConformanceError> {
    let report = &validated.report;
    if report.executor.declaration == ExecutorDeclaration::Mock {
        return Err(NativeConformanceError::MockExecutorCannotSatisfyNativeGate);
    }

    // Which program produced these observations, before what they say
    // about the target. A run whose executable is not identified is a
    // run about an unnamed program, and every row below would then be a
    // statement about nothing in particular (ADR-018).
    let expected =
        expected_provenance.ok_or(NativeConformanceError::ExpectedProvenanceUnavailable)?;
    validate_executor_provenance(&report.executor, expected)?;

    // Claims before rows. A row's disposition already accounts for its
    // claims, but naming the claim is what tells a reader which corner of
    // a dimension the run failed to establish.
    for row in &report.claims {
        if !row.required {
            continue;
        }
        match row.disposition {
            EvidenceDisposition::Passed => {}
            EvidenceDisposition::UnresolvedByDesign => {
                return Err(NativeConformanceError::RequiredClaimMissing(row.claim));
            }
            EvidenceDisposition::Failed | EvidenceDisposition::InfrastructureError => {
                return Err(if row.bearing_cases.is_empty() {
                    NativeConformanceError::RequiredClaimMissing(row.claim)
                } else {
                    NativeConformanceError::RequiredClaimFailed(row.claim)
                });
            }
        }
    }

    for row in &report.evidence {
        if row.plan != EvidencePlanClass::Required {
            continue;
        }
        let Some(requirement) = crate::vocabulary::evidence_requirement_from_name(&row.requirement)
        else {
            return Err(NativeConformanceError::EvidenceCensusMismatch);
        };
        match row.disposition {
            EvidenceDisposition::Passed => {}
            EvidenceDisposition::InfrastructureError => {
                return Err(NativeConformanceError::RequiredEvidenceInfrastructureError(
                    requirement,
                ));
            }
            EvidenceDisposition::Failed if row.cases == 0 => {
                return Err(NativeConformanceError::RequiredEvidenceMissing(requirement));
            }
            EvidenceDisposition::Failed | EvidenceDisposition::UnresolvedByDesign => {
                return Err(NativeConformanceError::RequiredEvidenceFailed(requirement));
            }
        }
    }

    // Every case, whatever it bears on. A failure filed under a row the
    // plan does not require is still a case whose observation was not
    // what the contract requires, and the canonical census is the
    // evidence subject as a whole rather than the union of its required
    // rows. This is the rule the prototype gate has always applied, and
    // the two gates disagreeing about it was the defect.
    //
    // A case whose expected result is rejection is `Passed` when the
    // target rejected it as expected: `compare` derives the status from
    // the fixture's own expectation, so this loop asks for agreement
    // with the contract and not for acceptance by the target.
    for row in &report.cases {
        match row.status {
            CaseStatus::Passed => {}
            CaseStatus::Failed => {
                return Err(NativeConformanceError::NativeCaseFailed(row.case()));
            }
            CaseStatus::InfrastructureError => {
                return Err(NativeConformanceError::NativeCaseInfrastructureError(
                    row.case(),
                ));
            }
        }
    }

    // The report's own verdict on itself, last. The checks above name
    // the specific row, claim, or case that failed, and a reader is
    // better served by that than by the summary line they could have
    // read themselves; this arm is what catches a failed completeness
    // arising from anything the loops above do not enumerate. The set of
    // reports the gate accepts does not depend on the order — a
    // validated report's summary is recomputed from its own rows — so
    // the order is chosen for the quality of the refusal.
    if report.summary.completeness == ReportCompleteness::Failed {
        return Err(NativeConformanceError::ReportSummaryFailed);
    }

    Ok(())
}

/// Whether the observation is what the contract requires.
///
/// The verdict is compared always. A failure class is compared against
/// the set the contract admits, because the target reports one reason
/// for several reviewed causes and requiring the finest of them would
/// fail an honest executor over a distinction the target does not draw.
/// A stack and a resource figure are compared only where both sides have
/// one: the expectations are established statically, and a node that
/// reports no interpreter stack is not thereby disagreeing with them.
fn compare(fixture: &PrimitiveFixture, observed: &ObservedNativeOutcome) -> CaseStatus {
    if observed.verdict == NativeVerdict::InfrastructureError {
        return CaseStatus::InfrastructureError;
    }
    let expected = fixture.expected();
    let verdict_matched = match expected {
        ExpectedPrimitiveOutcome::Accept { .. } => observed.verdict == NativeVerdict::Accepted,
        ExpectedPrimitiveOutcome::Reject { classes, .. } => {
            observed.verdict == NativeVerdict::Rejected
                && observed
                    .observed_failure
                    .is_some_and(|class| classes.contains(&class))
        }
    };
    if verdict_matched
        && stacks_matched(expected, observed)
        && resources_agree(fixture.expected_resources(), &observed.resources)
    {
        CaseStatus::Passed
    } else {
        CaseStatus::Failed
    }
}

/// Whether a reported stack is the one the fixture states.
///
/// Vacuously true where the executor reports none or the fixture states
/// none. Neither absence is a disagreement, and treating one as a
/// failure would refuse every executor that validates transactions
/// rather than instrumenting an interpreter.
fn stacks_matched(expected: &ExpectedPrimitiveOutcome, observed: &ObservedNativeOutcome) -> bool {
    let main = match (
        expected.static_final_stack(),
        observed.final_stack.as_deref(),
    ) {
        (Some(stated), Some(reported)) => stated == reported,
        _ => true,
    };
    let alternate = match (
        expected.static_final_altstack(),
        observed.final_altstack.as_deref(),
    ) {
        (Some(stated), Some(reported)) => stated == reported,
        _ => true,
    };
    main && alternate
}

/// Whether the reported figures are the ones the fixture fixes.
///
/// Only the exact rows are compared, and an exact row disagreeing means
/// the executor ran something other than what it was handed — which is a
/// failure of the case, not a resource note.
pub(crate) fn resources_agree(
    expected: ExpectedResourceObservation,
    observed: &NativeResourceObservation,
) -> bool {
    let rows = [
        (expected.script_bytes, Some(observed.script_bytes)),
        (
            expected.initial_stack_items,
            Some(observed.initial_stack_items),
        ),
        (expected.peak_stack_items, observed.peak_stack_items),
        (expected.peak_altstack_items, observed.peak_altstack_items),
        (
            expected.maximum_element_bytes,
            observed.maximum_element_bytes,
        ),
        (
            expected.validation_budget_used,
            observed.validation_budget_used,
        ),
        (expected.transaction_weight, observed.transaction_weight),
    ];
    rows.into_iter().all(|(stated, reported)| {
        match (stated, reported) {
            (ResourceExpectation::Exact(fixed), Some(seen)) => fixed == seen,
            // Recorded-only, or a figure this executor cannot observe.
            _ => true,
        }
    })
}

/// Which requirements one case bears on.
///
/// The complete fixture rather than its case identity: a case identity
/// carries a group and an opcode, and the enforcement layer a case is
/// stated at — which is what decides between the two resource rows —
/// lives in the fixture. Reading the layer from anywhere else would be
/// reading a value the fixture did not state.
fn bearing_requirements(fixture: &PrimitiveFixture) -> Vec<TargetEvidenceRequirementId> {
    let case = fixture.case();
    let mut requirements = requirements_of(case.group(), fixture.enforcement_layer()).to_vec();
    if case.opcode().is_some()
        && !requirements.contains(&TargetEvidenceRequirementId::OpcodeSemantics)
    {
        requirements.push(TargetEvidenceRequirementId::OpcodeSemantics);
    }
    requirements
}

/// The typed claim rows, in claim order.
///
/// A claim the registry requires and no passing case bears on is
/// `Failed`, and a claim the registry does not require and no case bears
/// on is `UnresolvedByDesign` — which is neither a pass nor a failure but
/// the project stating which corner of a dimension it has not
/// established. A claim outside the required plan that a case *did*
/// establish still passes: the run gets credit for what it did.
fn claim_rows(
    registry: &ClaimRegistry,
    plan: &EvidencePlan,
    per_claim: &BTreeMap<NativeEvidenceClaim, Vec<(NativeCaseId, CaseStatus)>>,
) -> Vec<NativeEvidenceClaimResult> {
    registry
        .iter()
        .map(|record| {
            let bearing = per_claim
                .get(&record.claim())
                .map_or(&[][..], Vec::as_slice);
            // A claim is required only where its owning requirement is
            // itself required: the plan owns which dimensions this run
            // must establish, and the registry owns what establishing one
            // means.
            let required = record.is_required()
                && plan.class(record.requirement()) == Some(EvidencePlanClass::Required);
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

            NativeEvidenceClaimResult {
                claim: record.claim(),
                requirement: evidence_requirement_name(record.requirement())
                    .unwrap_or("unspelled_requirement")
                    .to_owned(),
                required,
                unresolved_reason: record.unresolved_reason().map(str::to_owned),
                bearing_cases: bearing.iter().map(|(case, _)| *case).collect(),
                disposition,
            }
        })
        .collect()
}

/// The evidence rows, in requirement order.
fn evidence_rows(
    plan: &EvidencePlan,
    registry: &ClaimRegistry,
    per_requirement: &BTreeMap<TargetEvidenceRequirementId, Vec<CaseStatus>>,
    claims: &[NativeEvidenceClaimResult],
) -> Vec<EvidenceRequirementResult> {
    let claim_disposition: BTreeMap<NativeEvidenceClaim, EvidenceDisposition> = claims
        .iter()
        .map(|row| (row.claim, row.disposition))
        .collect();

    plan.iter()
        .map(|(id, class)| {
            let statuses = per_requirement.get(&id).map_or(&[][..], Vec::as_slice);
            let required_claims = registry.required_claims(id);
            let owned_claims = registry.owned_by(id);

            // A required claim without a passing case is what stops one
            // corner of a dimension standing in for the whole of it.
            let missing_required_claims: BTreeSet<NativeEvidenceClaim> = required_claims
                .iter()
                .filter(|claim| claim_disposition.get(claim) != Some(&EvidenceDisposition::Passed))
                .copied()
                .collect();
            let unresolved_claims: BTreeSet<NativeEvidenceClaim> = owned_claims
                .iter()
                .filter(|claim| {
                    claim_disposition.get(claim) == Some(&EvidenceDisposition::UnresolvedByDesign)
                })
                .copied()
                .collect();

            let disposition = if statuses.contains(&CaseStatus::InfrastructureError) {
                EvidenceDisposition::InfrastructureError
            } else if statuses.contains(&CaseStatus::Failed) {
                EvidenceDisposition::Failed
            } else if statuses.is_empty() {
                // A required row with no case is a failure, not a
                // silence: the plan says the run must establish it.
                if class == EvidencePlanClass::Required {
                    EvidenceDisposition::Failed
                } else {
                    EvidenceDisposition::UnresolvedByDesign
                }
            } else if class == EvidencePlanClass::Required && !missing_required_claims.is_empty() {
                EvidenceDisposition::Failed
            } else {
                EvidenceDisposition::Passed
            };

            EvidenceRequirementResult {
                requirement: evidence_requirement_name(id)
                    .unwrap_or("unspelled_requirement")
                    .to_owned(),
                plan: class,
                disposition,
                cases: u32::try_from(statuses.len()).unwrap_or(u32::MAX),
                missing_required_claims,
                unresolved_claims,
            }
        })
        .collect()
}

/// The counts, and what they add up to.
fn summarize(
    cases: &[NativeCaseResult],
    evidence: &[EvidenceRequirementResult],
    claims: &[NativeEvidenceClaimResult],
) -> NativeReportSummary {
    let count = |wanted: CaseStatus| {
        u32::try_from(cases.iter().filter(|case| case.status == wanted).count()).unwrap_or(u32::MAX)
    };
    let required: Vec<&EvidenceRequirementResult> = evidence
        .iter()
        .filter(|row| row.plan == EvidencePlanClass::Required)
        .collect();
    let required_passed = required
        .iter()
        .filter(|row| row.disposition == EvidenceDisposition::Passed)
        .count();
    let unresolved = evidence
        .iter()
        .filter(|row| row.disposition == EvidenceDisposition::UnresolvedByDesign)
        .count();

    let required_claims: Vec<&NativeEvidenceClaimResult> =
        claims.iter().filter(|row| row.required).collect();
    let required_claims_passed = required_claims
        .iter()
        .filter(|row| row.disposition == EvidenceDisposition::Passed)
        .count();
    let claims_unresolved = claims
        .iter()
        .filter(|row| row.disposition == EvidenceDisposition::UnresolvedByDesign)
        .count();

    let cases_failed = count(CaseStatus::Failed);
    let cases_infrastructure_error = count(CaseStatus::InfrastructureError);
    let completeness = if required_passed != required.len()
        || required_claims_passed != required_claims.len()
        || cases_failed > 0
        || cases_infrastructure_error > 0
    {
        ReportCompleteness::Failed
    } else if unresolved > 0 || claims_unresolved > 0 {
        ReportCompleteness::PartialUnresolvedClaims
    } else {
        ReportCompleteness::CompleteForPrimitivePlan
    };

    NativeReportSummary {
        cases_total: u32::try_from(cases.len()).unwrap_or(u32::MAX),
        cases_passed: count(CaseStatus::Passed),
        cases_failed,
        cases_infrastructure_error,
        required_evidence_total: u32::try_from(required.len()).unwrap_or(u32::MAX),
        required_evidence_passed: u32::try_from(required_passed).unwrap_or(u32::MAX),
        evidence_unresolved_by_design: u32::try_from(unresolved).unwrap_or(u32::MAX),
        required_claims_total: u32::try_from(required_claims.len()).unwrap_or(u32::MAX),
        required_claims_passed: u32::try_from(required_claims_passed).unwrap_or(u32::MAX),
        claims_unresolved: u32::try_from(claims_unresolved).unwrap_or(u32::MAX),
        completeness,
    }
}

/// What the caller intended the environment to have active.
///
/// Crate-visible because the prototype report states the same record
/// from the same binding, and two copies of it would eventually disagree
/// about what a caller declared.
pub(crate) fn activation_record_of(binding: &ReviewedDevelopmentBinding) -> ActivationRecord {
    let activation = binding.binding().activation();
    ActivationRecord {
        tapscript_expected_active: activation.tapscript_expected_active(),
        required_leaf_version: activation.required_leaf_version().get(),
        required_capabilities: activation
            .required_capabilities()
            .iter()
            .map(|capability| {
                capability_name(*capability)
                    .unwrap_or("unspelled_capability")
                    .to_owned()
            })
            .collect(),
    }
}

/// Which runner produced the observations, and what it was declared to
/// be.
///
/// Crate-visible for the prototype report, which records provenance from
/// the same handshake under the same rule: five roles kept apart, and
/// every field still what the executor *says*.
pub(crate) fn provenance_of(transcript: &ExecutionTranscript) -> ExecutorProvenance {
    let handshake = transcript.handshake();
    ExecutorProvenance {
        protocol_schema: handshake.protocol_schema,
        adapter_name: handshake.adapter_name.clone(),
        adapter_version: handshake.adapter_version.clone(),
        framework_revision: handshake.framework_revision.clone(),
        node_name: handshake.node_name.clone(),
        node_version: handshake.node_version.clone(),
        binary_reported_revision: handshake.binary_reported_revision.clone(),
        intended_executed_tip: handshake.intended_executed_tip.clone(),
        upstream_base: handshake.upstream_base.clone(),
        included_local_topics: handshake.included_local_topics.clone(),
        supported_domains: handshake.supported_domains.clone(),
        supported_leaf_versions: handshake.supported_leaf_versions.clone(),
        capabilities: handshake.capabilities.clone(),
        declaration: match transcript.trust() {
            ExecutorTrust::Mock => ExecutorDeclaration::Mock,
            ExecutorTrust::ReviewedNonMock => ExecutorDeclaration::ReviewedNonMock,
        },
    }
}
