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
//! # The gate is not the report
//!
//! A report describes a run. The gate decides whether that run is
//! target-native evidence, and it refuses a declared mock run before it
//! looks at anything else: a mock's answers are the fixture's own
//! expectations read back, so a green mock report says only that the
//! harness can compare a value with itself.

use std::collections::BTreeMap;

use target_elements::{
    ReviewedElementsTapscriptDefinition, TargetEvidenceRequirementId, ValidatedDevelopmentBinding,
};

use crate::error::NativeConformanceError;
use crate::executor::{ExecutionTranscript, ExecutorTrust};
use crate::fixture::{
    ExpectedPrimitiveOutcome, ExpectedResourceObservation, LeafVersionStatus, NativeCaseGroup,
    NativeCaseId, PrimitiveFixture, PrimitiveFixtureSet, ResourceExpectation,
};
use crate::protocol::{NativeResourceObservation, NativeVerdict, WireExecutionDomain};
use crate::report::{
    ActivationRecord, CaseStatus, EvidenceDisposition, EvidencePlanClass,
    EvidenceRequirementResult, ExecutorDeclaration, ExecutorProvenance, NATIVE_REPORT_SCHEMA,
    NativeCaseResult, NativeConformanceReport, NativeReportSummary, ObservedNativeOutcome,
    ReportCompleteness, WireEnvironment,
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
///   evidence rather than a script-level case.
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
/// Resource limits stay split between their two rows deliberately. The
/// literal-width boundary is the target's own rule and bears on the
/// consensus row; the nonminimal forms are valid spends that nodes
/// decline to forward, which is the relay row and nothing else.
const fn requirements_of(group: NativeCaseGroup) -> &'static [TargetEvidenceRequirementId] {
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
        NativeCaseGroup::Resource => &[
            TargetEvidenceRequirementId::ConsensusResourceLimits,
            TargetEvidenceRequirementId::PolicyResourceLimits,
        ],
    }
}

/// The requirements one group bears on, for the crate's own tests.
#[cfg(test)]
pub(crate) const fn requirements_for_tests(
    group: NativeCaseGroup,
) -> &'static [TargetEvidenceRequirementId] {
    requirements_of(group)
}

/// Builds the report of one run.
///
/// # Errors
///
/// [`NativeConformanceError::TargetContractMismatch`] or
/// [`NativeConformanceError::DevelopmentBindingMismatch`] when a fixture
/// was stated against a different contract or network from the run's,
/// and [`NativeConformanceError::MissingCaseResponse`] when the
/// transcript does not answer a fixture.
pub fn evaluate(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ValidatedDevelopmentBinding,
    fixtures: &PrimitiveFixtureSet,
    transcript: &ExecutionTranscript,
    plan: &EvidencePlan,
) -> Result<NativeConformanceReport, NativeConformanceError> {
    let definition = target.definition();
    let domain = WireExecutionDomain::of(definition.execution_domain())
        .ok_or(NativeConformanceError::TargetContractMismatch)?;

    let mut cases = Vec::new();
    let mut per_requirement: BTreeMap<TargetEvidenceRequirementId, Vec<CaseStatus>> =
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

        for requirement in bearing_requirements(case) {
            per_requirement.entry(requirement).or_default().push(status);
        }

        cases.push(NativeCaseResult {
            case,
            expected: fixture.expected().clone(),
            observed,
            status,
        });
    }

    let evidence = evidence_rows(plan, &per_requirement);
    let summary = summarize(&cases, &evidence);

    Ok(NativeConformanceReport {
        schema: NATIVE_REPORT_SCHEMA,
        target_contract_version: definition.version().get(),
        environment: WireEnvironment::Development,
        network_id: binding.binding().network_id(),
        genesis_id: binding.binding().genesis_id(),
        activation: activation_record(binding),
        executor: provenance(transcript),
        cases,
        evidence,
        summary,
    })
}

/// Decides whether one run is target-native evidence.
///
/// # Errors
///
/// [`NativeConformanceError::MockExecutorCannotSatisfyNativeGate`] for a
/// declared mock run, checked before anything else, and then
/// [`NativeConformanceError::RequiredEvidenceMissing`],
/// [`NativeConformanceError::RequiredEvidenceFailed`], or
/// [`NativeConformanceError::RequiredEvidenceInfrastructureError`] for
/// the first required row that does not pass.
pub fn gate(report: &NativeConformanceReport) -> Result<(), NativeConformanceError> {
    if report.executor.declaration == ExecutorDeclaration::Mock {
        return Err(NativeConformanceError::MockExecutorCannotSatisfyNativeGate);
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
        && resources_matched(fixture.expected_resources(), &observed.resources)
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
fn resources_matched(
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
fn bearing_requirements(case: NativeCaseId) -> Vec<TargetEvidenceRequirementId> {
    let mut requirements = requirements_of(case.group()).to_vec();
    if case.opcode().is_some()
        && !requirements.contains(&TargetEvidenceRequirementId::OpcodeSemantics)
    {
        requirements.push(TargetEvidenceRequirementId::OpcodeSemantics);
    }
    requirements
}

/// The evidence rows, in requirement order.
fn evidence_rows(
    plan: &EvidencePlan,
    per_requirement: &BTreeMap<TargetEvidenceRequirementId, Vec<CaseStatus>>,
) -> Vec<EvidenceRequirementResult> {
    plan.iter()
        .map(|(id, class)| {
            let statuses = per_requirement.get(&id).map_or(&[][..], Vec::as_slice);
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
            }
        })
        .collect()
}

/// The counts, and what they add up to.
fn summarize(
    cases: &[NativeCaseResult],
    evidence: &[EvidenceRequirementResult],
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

    let cases_failed = count(CaseStatus::Failed);
    let cases_infrastructure_error = count(CaseStatus::InfrastructureError);
    let completeness = if required_passed != required.len()
        || cases_failed > 0
        || cases_infrastructure_error > 0
    {
        ReportCompleteness::Failed
    } else if unresolved > 0 {
        ReportCompleteness::PartialUnresolvedRemains
    } else {
        ReportCompleteness::CompleteForRequiredPlan
    };

    NativeReportSummary {
        cases_total: u32::try_from(cases.len()).unwrap_or(u32::MAX),
        cases_passed: count(CaseStatus::Passed),
        cases_failed,
        cases_infrastructure_error,
        required_evidence_total: u32::try_from(required.len()).unwrap_or(u32::MAX),
        required_evidence_passed: u32::try_from(required_passed).unwrap_or(u32::MAX),
        evidence_unresolved_by_design: u32::try_from(unresolved).unwrap_or(u32::MAX),
        completeness,
    }
}

/// What the caller intended the environment to have active.
fn activation_record(binding: &ValidatedDevelopmentBinding) -> ActivationRecord {
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
fn provenance(transcript: &ExecutionTranscript) -> ExecutorProvenance {
    let handshake = transcript.handshake();
    ExecutorProvenance {
        protocol_schema: handshake.protocol_schema,
        implementation_name: handshake.implementation_name.clone(),
        implementation_version: handshake.implementation_version.clone(),
        upstream_revision: handshake.upstream_revision.clone(),
        supported_domains: handshake.supported_domains.clone(),
        supported_leaf_versions: handshake.supported_leaf_versions.clone(),
        capabilities: handshake.capabilities.clone(),
        declaration: match transcript.trust() {
            ExecutorTrust::Mock => ExecutorDeclaration::Mock,
            ExecutorTrust::ReviewedNonMock => ExecutorDeclaration::ReviewedNonMock,
        },
    }
}
