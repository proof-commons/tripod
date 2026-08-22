//! The validated operation report §4.4 makes coverage discharge read.
//!
//! # What this module exists to remove
//!
//! Coverage used to move on a tuple. A caller stated a vector identity,
//! a layer, and a projection verdict, and rows went to discharged; every
//! value in that tuple was a public enum a consumer could name for
//! itself, so the counters a gate reads could be reached without an
//! executor, without a target binding, and without anyone performing the
//! §17.4 comparison. The tuple was the whole of the correspondence
//! between a run and its report, and a tuple is not a run.
//!
//! So the tuple-level entries are crate-private now, and the only route
//! from outside is [`validate_operation_report`], which consumes *both*
//! transcripts a run produces — the target-generic
//! [`ExecutionTranscript`] the executor assembled, and the
//! [`OperationTranscript`] the planner kept — and recomputes what binds
//! them to each other and to their subject. An
//! [`ExecutionTranscript`] has no public constructor, so a caller cannot
//! begin.
//!
//! # Recomputed here, never accepted
//!
//! Nothing a caller states enters a verdict. The projection comparison
//! is performed *in this module* out of the bytes the target took and
//! the coins the target said it created; the reviewed target contract is
//! rebuilt from this workspace's own definition rather than read off the
//! transcript; the two transcripts are matched against each other by the
//! submitted bytes rather than by a case name the planner chose. What is
//! left over — the facts the transcript records but cannot check — is
//! recorded as what the executor *said*, and named as such.

use std::collections::{BTreeMap, BTreeSet};

use target_elements::{DeploymentEnvironment, DeploymentProjection, TargetProjection};
use target_elements_conformance::executor::{ExecutionTranscript, ExecutorTrust};
use target_elements_conformance::protocol::{
    ObservedOutcomeLayer, OperationCaseId, OperationSubject, WireEnvironment,
};

use crate::comparison::{compare, read_accepted};
use crate::fixture::{OPERATION, positive_semantic_census};
use crate::materialize::{TargetVectorId, vector_id};
use crate::mutation::NegativeMutation;
use crate::operation::OperationTranscript;
use crate::plan::ProjectionComparison;

/// The schema of the canonical rendered operation report.
///
/// Stated in the bytes so a reader never has to infer which revision a
/// file is: a report whose field set changed under a reader that assumed
/// the old one would otherwise be read wrong rather than refused.
pub const OPERATION_REPORT_SCHEMA: u32 = 1;

/// Why a run could not be validated into a report.
///
/// Every variant is a contradiction between two records that describe
/// one run, or between a record and what this workspace recomputed. None
/// of them is a target verdict: a target that refused every transaction
/// produces a perfectly valid report saying so.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReportValidationRefusal {
    /// The planner refused the plan, so the run did not finish.
    ///
    /// A refused run is not a report of a completed one. Its transcript
    /// is still readable and still worth writing down; what it cannot do
    /// is discharge coverage, because the work the coverage is about was
    /// not carried out.
    PlanWasRefused,
    /// The reviewed target contract could not be rebuilt here.
    ReviewedTargetUnavailable,
    /// The run was requested under a different target contract than this
    /// workspace's own definition states.
    TargetContractDiffers,
    /// What the executor said it ran on is not what the run was bound to.
    ///
    /// The executor performs this comparison too. It is repeated here
    /// because a report that trusted the other side to have checked
    /// would establish only that the other side claims to have checked.
    EnvironmentDiffers,
    /// The ceremony never established the asset or the program, so no
    /// projection can be read out of any transaction.
    CeremonyIncomplete,
    /// Two steps submitted byte-identical transactions.
    ///
    /// The bytes are how the two transcripts are matched, so a
    /// duplicate makes the correspondence ambiguous rather than merely
    /// surprising.
    DuplicateSubmittedBytes,
    /// A step the executor was asked was never answered.
    StepWentUnanswered(OperationCaseId),
    /// The two transcripts disagree about how many transactions were
    /// submitted.
    SubmissionCensusDiffers {
        /// How many submission steps the executor recorded sending.
        executor: usize,
        /// How many the planner recorded the answers to.
        planner: usize,
    },
    /// The planner recorded a submission the executor never sent.
    SubmissionNotSent,
    /// The two transcripts disagree about where the target put a
    /// transaction they both hold the bytes of.
    OutcomeDiffers(TargetVectorId),
    /// The target accepted a transaction and named no identity for it.
    AcceptedWithoutIdentity(TargetVectorId),
    /// The two transcripts disagree about the identity the target gave a
    /// transaction.
    AcceptedIdentityDiffers(TargetVectorId),
    /// §20.5's comparison failed: the two sides weighed the same bytes
    /// differently.
    WeightObservationDisagrees(TargetVectorId),
    /// A submission names a vector the positive census does not contain.
    SubmissionNamesNoFixture(TargetVectorId),
}

/// What the §17.4 comparison found for one accepted transaction.
///
/// The verdict and the reason travel together. A reader that had only
/// the verdict could not tell an unreadable transaction from an
/// unaccepted one, and both of those are `NotPerformed`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionVerdict {
    comparison: ProjectionComparison,
    detail: String,
}

impl ProjectionVerdict {
    /// What the comparison found.
    #[must_use]
    pub const fn comparison(&self) -> ProjectionComparison {
        self.comparison
    }

    /// Why, in this package's own words.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// What the executor said about itself.
///
/// # Why this is recorded and not compared
///
/// ADR-018 assigns a provenance comparison to the gate: what the
/// operator declared the executor was built from, against what the
/// executor reports. The expectation lives on the executor
/// *configuration*, and a transcript does not carry its configuration —
/// so the comparison has no second operand here, and the conversion the
/// conformance package uses for it (`provenance_of`) is private to that
/// package.
///
/// The seam is left open deliberately rather than closed with a check
/// that compares the executor's report against itself. Closing it means
/// either the transcript retaining the expectation it was run under, or
/// the conformance package publishing its provenance view; until one of
/// those happens, what this type states is what the executor said, and
/// it says so in its own name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutorSelfDescription {
    adapter_name: String,
    adapter_version: String,
    node_name: String,
    node_version: String,
    intended_executed_tip: Option<String>,
}

impl ExecutorSelfDescription {
    /// What the adapter calls itself.
    #[must_use]
    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    /// What version the adapter calls itself.
    #[must_use]
    pub fn adapter_version(&self) -> &str {
        &self.adapter_version
    }

    /// What the executing node calls itself.
    #[must_use]
    pub fn node_name(&self) -> &str {
        &self.node_name
    }

    /// The node's own version line.
    #[must_use]
    pub fn node_version(&self) -> &str {
        &self.node_version
    }

    /// The integration tip the operator said was executed.
    ///
    /// Unverified here, for the reason this type's own documentation
    /// gives.
    #[must_use]
    pub fn intended_executed_tip(&self) -> Option<&str> {
        self.intended_executed_tip.as_deref()
    }
}

/// The facts one execution transcript contributes, once extracted.
///
/// Crate-private, and that is the whole point: the public constructor
/// takes an [`ExecutionTranscript`], which nothing outside the
/// conformance package can assemble, and this is the only other way to
/// reach the validation. The crate's own tests use it to stage a
/// disagreeing pair without a live node; a consumer cannot name the
/// type.
pub(crate) struct ExecutionBinding {
    pub(crate) target: TargetProjection,
    pub(crate) deployment: DeploymentProjection,
    pub(crate) environment_class: WireEnvironment,
    pub(crate) environment_network: [u8; 32],
    pub(crate) environment_genesis: [u8; 32],
    pub(crate) trust: ExecutorTrust,
    pub(crate) executor: ExecutorSelfDescription,
    pub(crate) requests: BTreeMap<OperationCaseId, OperationSubject>,
    pub(crate) responses: BTreeMap<OperationCaseId, ObservedAnswer>,
}

/// The half of an executor answer this validation compares against.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObservedAnswer {
    pub(crate) layer: ObservedOutcomeLayer,
    pub(crate) accepted_txid: Option<String>,
}

/// One run, validated: the only thing canonical coverage discharge
/// reads.
///
/// # What holding one of these establishes
///
/// That two independently kept records of the same run agree with each
/// other and with what this workspace recomputed from them: the run was
/// requested under this workspace's own reviewed target contract, the
/// executor reported running on the chain the run was bound to, every
/// transaction the planner recorded an answer for is a transaction the
/// executor recorded sending, byte for byte, and the answers agree
/// down to the identity the target gave each accepted transaction. The
/// §17.4 comparisons were performed here, over the target's own bytes
/// and the target's own coins.
///
/// It establishes nothing about what the target *decided*. A report
/// whose every submission was refused is as valid as one whose every
/// submission was accepted; the difference shows up in what the coverage
/// rows then say, which is where it belongs.
#[derive(Debug, Eq, PartialEq)]
pub struct ValidatedCompactAshOperationReport<'run> {
    transcript: &'run OperationTranscript,
    target: TargetProjection,
    deployment: DeploymentProjection,
    trust: ExecutorTrust,
    executor: ExecutorSelfDescription,
    projections: BTreeMap<TargetVectorId, ProjectionVerdict>,
    outcomes: Vec<(TargetVectorId, ObservedOutcomeLayer, ProjectionComparison)>,
    mutant_observations: Vec<(NegativeMutation, TargetVectorId, ObservedOutcomeLayer)>,
    steps_answered: usize,
    weights_compared: usize,
    weights_matched: usize,
}

impl<'run> ValidatedCompactAshOperationReport<'run> {
    /// The planner's own record of the validated run.
    ///
    /// Handed out whole because every field of it has now been checked
    /// against the executor's record. A renderer reads its detail from
    /// here rather than from a transcript nobody validated.
    #[must_use]
    pub const fn transcript(&self) -> &'run OperationTranscript {
        self.transcript
    }

    /// The reviewed target contract the run was requested under.
    #[must_use]
    pub const fn target(&self) -> &TargetProjection {
        &self.target
    }

    /// The deployment binding the run was requested under.
    #[must_use]
    pub const fn deployment(&self) -> &DeploymentProjection {
        &self.deployment
    }

    /// What the caller declared the executor to be.
    ///
    /// A declaration, not a finding: a mock declared reviewed is still a
    /// mock. It is carried so a reader of a report can see which claim
    /// the run was made under.
    #[must_use]
    pub const fn trust(&self) -> ExecutorTrust {
        self.trust
    }

    /// What the executor said about itself.
    #[must_use]
    pub const fn executor(&self) -> &ExecutorSelfDescription {
        &self.executor
    }

    /// The §17.4 verdict for every transaction the target accepted.
    ///
    /// Recomputed in this module. A vector the target did not accept has
    /// no entry at all — §1.4's second verdict does not exist for a
    /// transaction that never reached the first.
    #[must_use]
    pub const fn projections(&self) -> &BTreeMap<TargetVectorId, ProjectionVerdict> {
        &self.projections
    }

    /// How many steps the executor answered.
    #[must_use]
    pub const fn steps_answered(&self) -> usize {
        self.steps_answered
    }

    /// How many submissions §20.5's weight comparison could be made for.
    ///
    /// Deliberately not the submission count: an executor that observes
    /// no weight leaves the comparison unmade, and counting those as
    /// matched would turn an absent observation into a passing one.
    #[must_use]
    pub const fn weights_compared(&self) -> usize {
        self.weights_compared
    }

    /// How many of those comparisons agreed.
    ///
    /// Equal to [`Self::weights_compared`] for every report that exists:
    /// a disagreement refuses validation, because the two sides weighed
    /// the same bytes. The figure is reported anyway, so a reader sees
    /// the comparison was made rather than inferring it.
    #[must_use]
    pub const fn weights_matched(&self) -> usize {
        self.weights_matched
    }

    /// The outcomes coverage discharge reads, in submission order.
    pub(crate) fn outcomes(
        &self,
    ) -> &[(TargetVectorId, ObservedOutcomeLayer, ProjectionComparison)] {
        &self.outcomes
    }

    /// The mutation observations coverage discharge reads.
    pub(crate) fn mutant_observations(
        &self,
    ) -> &[(NegativeMutation, TargetVectorId, ObservedOutcomeLayer)] {
        &self.mutant_observations
    }
}

/// Validate one run into the report coverage discharge reads.
///
/// The two transcripts are the two records of one run: the executor kept
/// what it was asked and what it heard, and the planner kept what those
/// answers meant in this package's vocabulary. Neither is checkable
/// against itself, which is why both are required.
///
/// # Errors
///
/// [`ReportValidationRefusal`], naming the first contradiction found in
/// the order the checks are written. A refusal here is never a statement
/// about the target: it says the two records of the run do not describe
/// the same run, or that one of them contradicts what this workspace
/// recomputed.
pub fn validate_operation_report<'run>(
    execution: &ExecutionTranscript,
    planner: &'run OperationTranscript,
) -> Result<ValidatedCompactAshOperationReport<'run>, ReportValidationRefusal> {
    let handshake = execution.handshake();
    let observation = execution.environment();
    let binding = ExecutionBinding {
        target: execution.target().clone(),
        deployment: execution.deployment().clone(),
        environment_class: observation.environment,
        environment_network: observation.network_id,
        environment_genesis: observation.genesis_id,
        trust: execution.trust(),
        executor: ExecutorSelfDescription {
            adapter_name: handshake.adapter_name.clone(),
            adapter_version: handshake.adapter_version.clone(),
            node_name: handshake.node_name.clone(),
            node_version: handshake.node_version.clone(),
            intended_executed_tip: handshake.intended_executed_tip.clone(),
        },
        requests: execution.operation_requests().clone(),
        responses: execution
            .operation_responses()
            .iter()
            .map(|(case, response)| {
                (
                    case.clone(),
                    ObservedAnswer {
                        layer: response.observed_layer,
                        accepted_txid: response.accepted_txid.clone(),
                    },
                )
            })
            .collect(),
    };
    validate_bound_run(&binding, planner)
}

/// The validation proper, over the extracted binding.
///
/// Split from [`validate_operation_report`] so the crate's own tests can
/// stage a contradicting pair of records without a live node. The split
/// costs nothing in strength: [`ExecutionBinding`] is crate-private, so
/// the only way in from outside remains a real execution transcript.
pub(crate) fn validate_bound_run<'run>(
    binding: &ExecutionBinding,
    planner: &'run OperationTranscript,
) -> Result<ValidatedCompactAshOperationReport<'run>, ReportValidationRefusal> {
    // A refused plan first. Everything below compares two records of a
    // run that finished, and a run that stopped partway has answers for
    // some steps and none for the rest by design rather than by defect.
    if planner.refusal().is_some() {
        return Err(ReportValidationRefusal::PlanWasRefused);
    }
    check_subject(binding)?;

    let sent = submission_index(binding)?;
    // A mutation that could not be built was never sent, and carries no
    // bytes to say so with; it is excluded from the census on that fact
    // rather than on its layer.
    let submitted_mutants = planner
        .mutants()
        .iter()
        .filter(|mutant| !mutant.bytes().is_empty())
        .count();
    if sent.len() != planner.submissions().len() + submitted_mutants {
        return Err(ReportValidationRefusal::SubmissionCensusDiffers {
            executor: sent.len(),
            planner: planner.submissions().len() + submitted_mutants,
        });
    }

    let census = positive_semantic_census()
        .map_err(|_| ReportValidationRefusal::ReviewedTargetUnavailable)?;
    let (weights_compared, weights_matched) = check_submissions(binding, planner, &sent, &census)?;
    check_mutants(binding, planner, &sent)?;

    let projections = compare_projections(planner, &census)?;
    let outcomes = planner
        .submissions()
        .iter()
        .map(|submission| {
            let comparison = projections.get(&submission.vector()).map_or(
                ProjectionComparison::NotPerformed,
                ProjectionVerdict::comparison,
            );
            (submission.vector(), submission.layer(), comparison)
        })
        .collect();
    let mutant_observations = planner
        .mutants()
        .iter()
        .map(|mutant| (mutant.mutation(), mutant.origin(), mutant.layer()))
        .collect();

    Ok(ValidatedCompactAshOperationReport {
        transcript: planner,
        target: binding.target.clone(),
        deployment: binding.deployment.clone(),
        trust: binding.trust,
        executor: binding.executor.clone(),
        projections,
        outcomes,
        mutant_observations,
        steps_answered: binding.responses.len(),
        weights_compared,
        weights_matched,
    })
}

/// Whether the run was requested against the subject it claims.
///
/// The reviewed contract is rebuilt here rather than read off the
/// transcript, because reading it off the transcript would compare the
/// run's own claim with itself. The chain is settled inside the
/// transcript, between what the run was bound to and what the executor
/// said it ran on — the executor compares those two as well, and this
/// repeats the comparison rather than establishing only that the other
/// side claims to have made it.
///
/// # Errors
///
/// [`ReportValidationRefusal`], naming which of the three disagreed.
fn check_subject(binding: &ExecutionBinding) -> Result<(), ReportValidationRefusal> {
    let reviewed = target_elements::reviewed_elements_tapscript()
        .map_err(|_| ReportValidationRefusal::ReviewedTargetUnavailable)?;
    if reviewed.projection() != binding.target {
        return Err(ReportValidationRefusal::TargetContractDiffers);
    }

    let class_agrees = match binding.deployment.environment() {
        DeploymentEnvironment::Development => {
            binding.environment_class == WireEnvironment::Development
        }
        // No validated production binding exists, so this arm refuses
        // rather than choosing a development answer for a production
        // question.
        _ => false,
    };
    if !class_agrees
        || binding.environment_network != binding.deployment.network_id()
        || binding.environment_genesis != binding.deployment.genesis_id()
    {
        return Err(ReportValidationRefusal::EnvironmentDiffers);
    }

    for case in binding.requests.keys() {
        if !binding.responses.contains_key(case) {
            return Err(ReportValidationRefusal::StepWentUnanswered(case.clone()));
        }
    }
    Ok(())
}

/// Every transaction the executor recorded sending, by its bytes.
///
/// The two transcripts are matched on the submitted bytes, not on a step
/// name the planner chose for itself: a name is this package agreeing
/// with itself, and the bytes are what the target was actually handed.
///
/// # Errors
///
/// [`ReportValidationRefusal::DuplicateSubmittedBytes`] where two steps
/// sent the same transaction, which makes the correspondence ambiguous.
fn submission_index(
    binding: &ExecutionBinding,
) -> Result<BTreeMap<&[u8], &OperationCaseId>, ReportValidationRefusal> {
    let mut sent: BTreeMap<&[u8], &OperationCaseId> = BTreeMap::new();
    for (case, subject) in &binding.requests {
        if let OperationSubject::Submission(submission) = subject
            && sent
                .insert(submission.transaction_bytes.as_slice(), case)
                .is_some()
        {
            return Err(ReportValidationRefusal::DuplicateSubmittedBytes);
        }
    }
    Ok(sent)
}

/// Hold every recorded submission against the executor's own answer.
///
/// Returns how many §20.5 weight comparisons could be made and how many
/// agreed. The second figure equals the first for every report that
/// exists, because a disagreement refuses here; both are returned so a
/// reader sees the comparison was made rather than inferring it.
///
/// # Errors
///
/// [`ReportValidationRefusal`], naming the first submission whose two
/// records disagree, whose vector the census does not contain, whose
/// acceptance carries no identity, or whose weight was settled twice to
/// two different answers.
fn check_submissions(
    binding: &ExecutionBinding,
    planner: &OperationTranscript,
    sent: &BTreeMap<&[u8], &OperationCaseId>,
    census: &[crate::fixture::CompactAshSemanticCase],
) -> Result<(usize, usize), ReportValidationRefusal> {
    let known: BTreeSet<TargetVectorId> = census.iter().map(vector_id).collect();
    let mut compared = 0_usize;
    let mut matched = 0_usize;
    for submission in planner.submissions() {
        let vector = submission.vector();
        if !known.contains(&vector) {
            return Err(ReportValidationRefusal::SubmissionNamesNoFixture(vector));
        }
        let answer = answer_for(binding, sent, submission.bytes())?;
        if answer.layer != submission.layer() {
            return Err(ReportValidationRefusal::OutcomeDiffers(vector));
        }
        if answer.accepted_txid.as_deref() != submission.accepted_txid() {
            return Err(ReportValidationRefusal::AcceptedIdentityDiffers(vector));
        }
        if submission.layer() == ObservedOutcomeLayer::Accepted
            && submission
                .accepted_txid()
                .is_none_or(|txid| txid.trim().is_empty())
        {
            return Err(ReportValidationRefusal::AcceptedWithoutIdentity(vector));
        }
        // §20.5. The two sides settled a weight from the same bytes, so
        // a disagreement is not a tolerance to widen; the planner
        // refuses such a run outright, and a report assembled by some
        // other route is held to the same bar here.
        match submission.weight_agrees() {
            Some(true) => {
                compared += 1;
                matched += 1;
            }
            Some(false) => {
                return Err(ReportValidationRefusal::WeightObservationDisagrees(vector));
            }
            None => {}
        }
    }
    Ok((compared, matched))
}

/// The negative half, held to the same correspondence.
///
/// A mutation carrying no bytes was never submitted — it could not be
/// built from the accepted shape — so it is skipped rather than sought
/// on the other side.
///
/// # Errors
///
/// [`ReportValidationRefusal`], naming the origin of the first mutation
/// whose two records disagree.
fn check_mutants(
    binding: &ExecutionBinding,
    planner: &OperationTranscript,
    sent: &BTreeMap<&[u8], &OperationCaseId>,
) -> Result<(), ReportValidationRefusal> {
    for mutant in planner.mutants() {
        if mutant.bytes().is_empty() {
            continue;
        }
        let answer = answer_for(binding, sent, mutant.bytes())?;
        if answer.layer != mutant.layer() {
            return Err(ReportValidationRefusal::OutcomeDiffers(mutant.origin()));
        }
        if answer.accepted_txid.as_deref() != mutant.accepted_txid() {
            return Err(ReportValidationRefusal::AcceptedIdentityDiffers(
                mutant.origin(),
            ));
        }
    }
    Ok(())
}

/// The executor's answer to the step that sent exactly these bytes.
///
/// # Errors
///
/// [`ReportValidationRefusal::SubmissionNotSent`] where the planner
/// recorded an answer to a transaction the executor never sent, and
/// [`ReportValidationRefusal::StepWentUnanswered`] where it sent one and
/// recorded no answer.
fn answer_for<'binding>(
    binding: &'binding ExecutionBinding,
    sent: &BTreeMap<&[u8], &OperationCaseId>,
    bytes: &[u8],
) -> Result<&'binding ObservedAnswer, ReportValidationRefusal> {
    let Some(case) = sent.get(bytes) else {
        return Err(ReportValidationRefusal::SubmissionNotSent);
    };
    binding
        .responses
        .get(*case)
        .ok_or_else(|| ReportValidationRefusal::StepWentUnanswered((*case).clone()))
}

/// §17.4's comparison, for every transaction the target accepted.
///
/// Performed here because it needs both halves and neither transcript
/// holds both: the expectation comes from the realization layer's own
/// arithmetic over the fixture, and the observation is read back out of
/// the bytes the target took and the coins the target said it created.
/// A run that reached no acceptance produces an empty map, which is a
/// different fact from a run whose comparisons all failed.
fn compare_projections(
    planner: &OperationTranscript,
    census: &[crate::fixture::CompactAshSemanticCase],
) -> Result<BTreeMap<TargetVectorId, ProjectionVerdict>, ReportValidationRefusal> {
    let mut verdicts = BTreeMap::new();
    let accepted = planner
        .submissions()
        .iter()
        .filter(|submission| submission.layer() == ObservedOutcomeLayer::Accepted)
        .count();
    if accepted == 0 {
        return Ok(verdicts);
    }

    // A run that accepted something must have established both, because
    // both had to exist before anything could be built to submit.
    let (Some(program), Some(asset)) = (planner.constructor_program(), planner.issued_asset())
    else {
        return Err(ReportValidationRefusal::CeremonyIncomplete);
    };

    for submission in planner.submissions() {
        if submission.layer() != ObservedOutcomeLayer::Accepted {
            continue;
        }
        let id = submission.vector();
        let Some(case) = census.iter().find(|case| vector_id(case) == id) else {
            return Err(ReportValidationRefusal::SubmissionNamesNoFixture(id));
        };
        // Only the coins this vector was funded with. Handing over every
        // coin the run created would let one vector's input read as
        // another's, and the sponsor count is derived from exactly the
        // inputs that are not in this set.
        let mine: BTreeMap<_, _> = planner
            .funded()
            .get(&id)
            .map(|outpoints| {
                outpoints
                    .iter()
                    .filter_map(|outpoint| {
                        planner
                            .coins()
                            .get(outpoint)
                            .map(|amount| (*outpoint, *amount))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let verdict = match read_accepted(submission.bytes(), &mine, asset, program, OPERATION) {
            // A transaction that could not be read has not disagreed
            // about anything, so the comparison was not performed. It
            // must not be recorded as a difference, which would file a
            // reading failure as a protocol finding.
            Err(refusal) => ProjectionVerdict {
                comparison: ProjectionComparison::NotPerformed,
                detail: format!("unreadable: {refusal:?}"),
            },
            Ok(observed) => {
                let differed = compare(case.expected(), &observed);
                if differed.is_empty() {
                    ProjectionVerdict {
                        comparison: ProjectionComparison::Matched,
                        detail: "matched".to_owned(),
                    }
                } else {
                    let names: Vec<&str> = differed.iter().map(|term| term.name()).collect();
                    ProjectionVerdict {
                        comparison: ProjectionComparison::Differed,
                        detail: format!("differed: {}", names.join(" ")),
                    }
                }
            }
        };
        verdicts.insert(id, verdict);
    }
    Ok(verdicts)
}

#[cfg(test)]
mod tests {
    use super::{
        ExecutionBinding, ExecutorSelfDescription, ObservedAnswer, ReportValidationRefusal,
        validate_bound_run,
    };
    use crate::fixture::positive_semantic_census;
    use crate::materialize::{TargetVectorId, vector_id};
    use crate::mutation::NegativeMutation;
    use crate::operation::{
        MutantOutcome, OperationTranscript, PlanRefusal, SubmissionOutcome, TranscriptParts,
    };
    use crate::plan::ProjectionComparison;
    use std::collections::BTreeMap;
    use target_elements::{
        ActivationDeclaration, DeploymentEnvironment, DeploymentProjection,
        DevelopmentDeploymentBinding, LeafVersion, TargetProjection, reviewed_elements_tapscript,
        validate_reviewed_development_binding,
    };
    use target_elements_conformance::executor::ExecutorTrust;
    use target_elements_conformance::protocol::{
        ObservedOutcomeLayer, OperationCaseId, OperationStepKind, OperationSubject,
        TargetSubmissionSubject, WireEnvironment,
    };

    const NETWORK: [u8; 32] = [0x11; 32];
    const GENESIS: [u8; 32] = [0x22; 32];

    /// The reviewed contract, as the validation recomputes it.
    fn target() -> TargetProjection {
        reviewed_elements_tapscript()
            .expect("the reviewed target validates")
            .projection()
    }

    /// A development binding the staged run is stated under.
    fn deployment() -> DeploymentProjection {
        let reviewed = reviewed_elements_tapscript().expect("the reviewed target validates");
        validate_reviewed_development_binding(
            &reviewed,
            DevelopmentDeploymentBinding::new(
                reviewed.definition().version(),
                DeploymentEnvironment::Development,
                NETWORK,
                GENESIS,
                ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
                None,
            ),
        )
        .expect("the development binding validates")
        .projection()
    }

    /// The first vector of the positive census, by its own identity.
    fn some_vector() -> TargetVectorId {
        let census = positive_semantic_census().expect("the positive census builds");
        vector_id(&census[0])
    }

    /// One submission step, as the executor would have recorded it.
    fn submission_step(step: &str, bytes: &[u8]) -> (OperationCaseId, OperationSubject) {
        (
            OperationCaseId {
                operation: OperationStepKind::Submit,
                step: step.to_owned(),
            },
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: bytes.to_vec(),
            })),
        )
    }

    /// A binding whose records answer `steps` in order.
    fn binding(
        steps: Vec<(OperationCaseId, OperationSubject)>,
        answers: Vec<ObservedAnswer>,
    ) -> ExecutionBinding {
        let responses = steps
            .iter()
            .map(|(case, _)| case.clone())
            .zip(answers)
            .collect::<BTreeMap<_, _>>();
        ExecutionBinding {
            target: target(),
            deployment: deployment(),
            environment_class: WireEnvironment::Development,
            environment_network: NETWORK,
            environment_genesis: GENESIS,
            trust: ExecutorTrust::Mock,
            executor: ExecutorSelfDescription {
                adapter_name: "staged".to_owned(),
                adapter_version: "0".to_owned(),
                node_name: "staged".to_owned(),
                node_version: "0".to_owned(),
                intended_executed_tip: None,
            },
            requests: steps.into_iter().collect(),
            responses,
        }
    }

    fn answer(layer: ObservedOutcomeLayer, txid: Option<&str>) -> ObservedAnswer {
        ObservedAnswer {
            layer,
            accepted_txid: txid.map(str::to_owned),
        }
    }

    /// A planner transcript carrying one accepted submission.
    fn accepted_run(bytes: &[u8], observed_weight: Option<u64>) -> OperationTranscript {
        OperationTranscript::for_tests(TranscriptParts {
            issued_asset: Some([0x33; 32]),
            constructor_program: Some(vec![0x51, 0x20]),
            submissions: vec![SubmissionOutcome::for_tests(
                some_vector(),
                ObservedOutcomeLayer::Accepted,
                Some("aa"),
                bytes.to_vec(),
                100,
                observed_weight,
            )],
            mutants: Vec::new(),
            refusal: None,
        })
    }

    /// `G13-R01`: an acceptance a caller states is not a matched
    /// projection.
    ///
    /// The whole of what the old tuple could do, staged as far as the
    /// crate's own boundary allows: a submission recorded as accepted,
    /// with an identity, matching the executor's answer exactly. The
    /// validation admits the run — the two records really do agree — and
    /// the §17.4 verdict is still `NotPerformed`, because the comparison
    /// is performed here out of the submitted bytes and those bytes are
    /// not a transaction. No route reaches `Matched` without a
    /// transaction that projects.
    #[test]
    fn an_accepted_submission_earns_no_projection_it_cannot_be_read_for() {
        let bytes = b"this is not a target transaction".to_vec();
        let planner = accepted_run(&bytes, None);
        let binding = binding(
            vec![submission_step("submit/0", &bytes)],
            vec![answer(ObservedOutcomeLayer::Accepted, Some("aa"))],
        );

        let report = validate_bound_run(&binding, &planner).expect("the two records agree");
        let verdict = report
            .projections()
            .get(&some_vector())
            .expect("an accepted submission is compared");
        assert_eq!(verdict.comparison(), ProjectionComparison::NotPerformed);
        assert!(verdict.detail().starts_with("unreadable:"));
        assert_eq!(report.outcomes().len(), 1);
        assert_eq!(
            report.outcomes()[0],
            (
                some_vector(),
                ObservedOutcomeLayer::Accepted,
                ProjectionComparison::NotPerformed
            )
        );
    }

    /// `G13-R01`: a submission the executor never sent is refused.
    ///
    /// The forgery the tuple made free: a planner record naming a
    /// submission with no counterpart on the other side of the boundary.
    /// The two transcripts are matched on the submitted bytes, so the
    /// claim has nothing to match against.
    #[test]
    fn a_submission_the_executor_never_sent_refuses_the_report() {
        let planner = accepted_run(b"invented", None);
        let binding = binding(Vec::new(), Vec::new());
        assert_eq!(
            validate_bound_run(&binding, &planner),
            Err(ReportValidationRefusal::SubmissionCensusDiffers {
                executor: 0,
                planner: 1,
            })
        );
    }

    /// `G13-R01`: the two records must agree about the target's verdict.
    ///
    /// A planner record saying accepted where the executor recorded a
    /// script-path refusal is not a disagreement to average: one of the
    /// two is wrong about an answer they both hold.
    #[test]
    fn two_records_disagreeing_about_the_verdict_refuse_the_report() {
        let bytes = b"the same bytes".to_vec();
        let planner = accepted_run(&bytes, None);
        let binding = binding(
            vec![submission_step("submit/0", &bytes)],
            vec![answer(ObservedOutcomeLayer::ScriptPathRejection, None)],
        );
        assert_eq!(
            validate_bound_run(&binding, &planner),
            Err(ReportValidationRefusal::OutcomeDiffers(some_vector()))
        );
    }

    /// `G13-R01`: an acceptance with no identity is refused.
    #[test]
    fn an_acceptance_the_target_named_no_identity_for_refuses_the_report() {
        let bytes = b"unnamed".to_vec();
        let planner = OperationTranscript::for_tests(TranscriptParts {
            issued_asset: Some([0x33; 32]),
            constructor_program: Some(vec![0x51, 0x20]),
            submissions: vec![SubmissionOutcome::for_tests(
                some_vector(),
                ObservedOutcomeLayer::Accepted,
                None,
                bytes.clone(),
                100,
                None,
            )],
            mutants: Vec::new(),
            refusal: None,
        });
        let binding = binding(
            vec![submission_step("submit/0", &bytes)],
            vec![answer(ObservedOutcomeLayer::Accepted, None)],
        );
        assert_eq!(
            validate_bound_run(&binding, &planner),
            Err(ReportValidationRefusal::AcceptedWithoutIdentity(
                some_vector()
            ))
        );
    }

    /// `G13-R01`: §20.5's disagreement refuses the report too.
    ///
    /// The planner refuses such a run outright (`G13-R03`), so a
    /// transcript reaching here with one came by some other route. It is
    /// held to the same bar: the two sides weighed the same bytes.
    #[test]
    fn a_weight_disagreement_refuses_the_report() {
        let bytes = b"weighed twice".to_vec();
        let planner = accepted_run(&bytes, Some(101));
        let binding = binding(
            vec![submission_step("submit/0", &bytes)],
            vec![answer(ObservedOutcomeLayer::Accepted, Some("aa"))],
        );
        assert_eq!(
            validate_bound_run(&binding, &planner),
            Err(ReportValidationRefusal::WeightObservationDisagrees(
                some_vector()
            ))
        );
    }

    /// An agreeing weight is counted, so the comparison is visibly made.
    #[test]
    fn an_agreeing_weight_is_reported_as_compared_and_matched() {
        let bytes = b"weighed once".to_vec();
        let planner = accepted_run(&bytes, Some(100));
        let binding = binding(
            vec![submission_step("submit/0", &bytes)],
            vec![answer(ObservedOutcomeLayer::Accepted, Some("aa"))],
        );
        let report = validate_bound_run(&binding, &planner).expect("the two records agree");
        assert_eq!(report.weights_compared(), 1);
        assert_eq!(report.weights_matched(), 1);
        assert_eq!(report.steps_answered(), 1);
        assert_eq!(report.trust(), ExecutorTrust::Mock);
    }

    /// `G13-R01`: a refused run is not a report.
    #[test]
    fn a_refused_plan_yields_no_validated_report() {
        let planner = OperationTranscript::for_tests(TranscriptParts {
            issued_asset: None,
            constructor_program: None,
            submissions: Vec::new(),
            mutants: Vec::new(),
            refusal: Some(PlanRefusal::IssuanceNamedNoAsset),
        });
        assert_eq!(
            validate_bound_run(&binding(Vec::new(), Vec::new()), &planner),
            Err(ReportValidationRefusal::PlanWasRefused)
        );
    }

    /// The chain the executor says it ran on must be the bound one.
    #[test]
    fn an_executor_reporting_another_chain_refuses_the_report() {
        let planner = OperationTranscript::for_tests(TranscriptParts {
            issued_asset: None,
            constructor_program: None,
            submissions: Vec::new(),
            mutants: Vec::new(),
            refusal: None,
        });
        let mut binding = binding(Vec::new(), Vec::new());
        binding.environment_genesis = [0x44; 32];
        assert_eq!(
            validate_bound_run(&binding, &planner),
            Err(ReportValidationRefusal::EnvironmentDiffers)
        );
    }

    /// A mutation the run submitted must appear on the executor's side.
    ///
    /// The negative half is matched on bytes exactly as the positive
    /// half is, so a refusal nobody submitted has nothing to match.
    #[test]
    fn a_mutation_the_executor_never_sent_refuses_the_report() {
        let bytes = b"a real submission".to_vec();
        let planner = OperationTranscript::for_tests(TranscriptParts {
            issued_asset: Some([0x33; 32]),
            constructor_program: Some(vec![0x51, 0x20]),
            submissions: vec![SubmissionOutcome::for_tests(
                some_vector(),
                ObservedOutcomeLayer::ScriptPathRejection,
                None,
                bytes.clone(),
                100,
                None,
            )],
            mutants: vec![MutantOutcome::for_tests(
                some_vector(),
                NegativeMutation::SplitSuccessorInTwo,
                ObservedOutcomeLayer::ScriptPathRejection,
                b"never sent".to_vec(),
            )],
            refusal: None,
        });
        let binding = binding(
            vec![submission_step("submit/0", &bytes)],
            vec![answer(ObservedOutcomeLayer::ScriptPathRejection, None)],
        );
        assert_eq!(
            validate_bound_run(&binding, &planner),
            Err(ReportValidationRefusal::SubmissionCensusDiffers {
                executor: 1,
                planner: 2,
            })
        );
    }

    /// A mutation both records hold, agreeing, reaches the negative
    /// observations the plan reads.
    #[test]
    fn an_agreeing_mutation_reaches_the_negative_observations() {
        let subject = b"the control".to_vec();
        let mutated = b"the mutation".to_vec();
        let planner = OperationTranscript::for_tests(TranscriptParts {
            issued_asset: Some([0x33; 32]),
            constructor_program: Some(vec![0x51, 0x20]),
            submissions: vec![SubmissionOutcome::for_tests(
                some_vector(),
                ObservedOutcomeLayer::ScriptPathRejection,
                None,
                subject.clone(),
                100,
                None,
            )],
            mutants: vec![MutantOutcome::for_tests(
                some_vector(),
                NegativeMutation::SplitSuccessorInTwo,
                ObservedOutcomeLayer::ScriptPathRejection,
                mutated.clone(),
            )],
            refusal: None,
        });
        let binding = binding(
            vec![
                submission_step("submit/0", &subject),
                submission_step("submit-mutant/0", &mutated),
            ],
            vec![
                answer(ObservedOutcomeLayer::ScriptPathRejection, None),
                answer(ObservedOutcomeLayer::ScriptPathRejection, None),
            ],
        );
        let report = validate_bound_run(&binding, &planner).expect("the two records agree");
        assert_eq!(report.mutant_observations().len(), 1);
        assert_eq!(
            report.mutant_observations()[0],
            (
                NegativeMutation::SplitSuccessorInTwo,
                some_vector(),
                ObservedOutcomeLayer::ScriptPathRejection
            )
        );
        // Nothing was accepted, so no §17.4 comparison exists at all.
        assert!(report.projections().is_empty());
    }

    /// Two steps submitting the same bytes make the correspondence
    /// ambiguous, and ambiguity is refused rather than resolved.
    #[test]
    fn two_steps_submitting_identical_bytes_refuse_the_report() {
        let bytes = b"identical".to_vec();
        let planner = accepted_run(&bytes, None);
        let binding = binding(
            vec![
                submission_step("submit/0", &bytes),
                submission_step("submit/1", &bytes),
            ],
            vec![
                answer(ObservedOutcomeLayer::Accepted, Some("aa")),
                answer(ObservedOutcomeLayer::Accepted, Some("aa")),
            ],
        );
        assert_eq!(
            validate_bound_run(&binding, &planner),
            Err(ReportValidationRefusal::DuplicateSubmittedBytes)
        );
    }
}
