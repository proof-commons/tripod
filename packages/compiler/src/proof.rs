//! Relation obligations and exact feasible proof-plan enumeration
//! (Guide-3 Tranches G–I).
//!
//! Every in-scope relation is classified exactly once: proof-required
//! (with its realization-approved alternatives), statically validated,
//! or an external evidence requirement retained unresolved. The exact
//! search then enumerates, depth-first and deterministically, every
//! complete assignment of one proof per proof-required relation and
//! one representation per representation variable, keeping the whole
//! feasible set — hard constraints prune, nothing is weighted, no
//! greedy fallback exists, and complexity exhaustion returns no
//! partial result. Search-state counts live in a diagnostic report,
//! never in a candidate.

use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
use crate::{CompileError, input::BoundCompilerInput};

use crate::{
    capability::{CapabilityView, RequiredCapability},
    constructibility::{CompilerConstructibilityAnalysis, validate_source_constructibility},
    disclosure::CompilerDisclosureAnalysis,
    lifecycle::{LifecycleRequirement, RepresentationChoiceId},
    search_counter::record_search_event,
    source::{OperandId, SourceRequirement, derive_source_requirements, proof_capabilities},
};
use realization::{ProofAlternativeId, RelationDeclaration, RelationId, RepresentationMode};

/// How one relation is discharged.
///
/// An externally evidenced relation is not a proof variable, but it is
/// still planned: it carries the realization-approved proof class it
/// will be discharged under, together with the capabilities and source
/// rows that class fixes. Those requirements are not optional, so they
/// bind before the search rather than inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationObligationClass {
    ProofRequired {
        alternatives: Vec<ProofAlternativeId>,
    },
    StaticallyValidated,
    ExternalEvidence {
        requirement: realization::ExternalEvidenceRequirement,
        proof: ProofAlternativeId,
        required_capabilities: BTreeSet<RequiredCapability>,
        source_requirements: Vec<SourceRequirement>,
    },
}

/// One relation's discharge obligation with its operand census.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationObligation {
    pub relation: RelationId,
    pub class: RelationObligationClass,
    pub operands: Vec<OperandId>,
}

/// One complete feasible plan.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProofPlanCandidate {
    pub proofs: BTreeMap<RelationId, ProofAlternativeId>,
    pub representations: BTreeMap<RepresentationChoiceId, RepresentationMode>,
    pub required_capabilities: BTreeSet<RequiredCapability>,
    pub source_requirements: Vec<SourceRequirement>,
    pub external_evidence: BTreeSet<realization::ExternalEvidenceRequirement>,
    pub lifecycle: Vec<LifecycleRequirement>,
    pub disclosure: CompilerDisclosureAnalysis,
}

/// Diagnostic search statistics — never semantic identity.
///
/// Every rejection counter names the cause it counts, and only that
/// cause. A report that folded several distinct causes into one counter
/// would still be a typed account of how analysis was performed, and it
/// would be a false one.
///
/// No counter is carried for a rejection the search cannot perform. A
/// selected representation's lifecycle requirements are filtered into
/// the candidate rather than tested against it, so lifecycle rejection
/// has no site here; a permanently zero counter would read as evidence
/// that the search looked and found nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProofSearchReport {
    pub states_visited: u64,
    pub complete_assignments: u64,
    pub feasible_assignments: u64,
    pub rejected_by_capability: u64,
    pub rejected_by_source: u64,
    pub rejected_by_constructibility: u64,
    pub rejected_by_representation: u64,
    pub rejected_by_disclosure: u64,
}

impl ProofSearchReport {
    /// Record one local rejection under the cause that produced it.
    pub const fn record_local_rejection(&mut self, rejection: LocalProofRejection) {
        let counter = match rejection {
            LocalProofRejection::MissingCapability => &mut self.rejected_by_capability,
            LocalProofRejection::SourceDerivation => &mut self.rejected_by_source,
            LocalProofRejection::Constructibility => &mut self.rejected_by_constructibility,
        };

        record_search_event(counter);
    }
}

/// Why one proof alternative failed the local feasibility filters.
///
/// The three causes are genuinely different questions about the same
/// alternative — whether the target can express the proof, whether the
/// proof's source rows can be derived at all, and whether the derived
/// rows can actually be constructed from the operation's witnesses — so
/// collapsing them loses the only information the rejection carried.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalProofRejection {
    /// The capability view does not support the proof's capabilities.
    MissingCapability,
    /// The proof's source requirements could not be derived.
    SourceDerivation,
    /// The derived source rows are not constructible for the relation.
    Constructibility,
}

/// The complete canonical feasible set.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg(test)]
pub struct FeasibleProofPlans {
    pub candidates: Vec<ProofPlanCandidate>,
    pub search: ProofSearchReport,
}

pub use crate::engine_attestation::proof::classify_obligations;

/// Enumerate the complete feasible plan set for one bound input.
#[cfg(test)]
pub fn enumerate_feasible_plans(
    input: &BoundCompilerInput,
    view: &CapabilityView,
) -> Result<FeasibleProofPlans, CompileError> {
    let limits = input.policy().proof_search_limits;
    let span = tracing::debug_span!(target: "compiler::proof", "enumerate_feasible_plans", state_limit = limits.maximum_states.get(), candidate_limit = limits.maximum_candidates.get());
    let _entered = span.enter();
    let mut search = ProofSearchReport::default();
    let result = enumerate_feasible_plans_core(input, view, &mut search);
    match &result {
        Ok(_) => {
            tracing::debug!(target: "compiler::proof", outcome = "completed", visited_state_count = search.states_visited, complete_assignment_count = search.complete_assignments, feasible_assignment_count = search.feasible_assignments, capability_rejection_count = search.rejected_by_capability, source_rejection_count = search.rejected_by_source, constructibility_rejection_count = search.rejected_by_constructibility, representation_rejection_count = search.rejected_by_representation, disclosure_rejection_count = search.rejected_by_disclosure, message = "proof search completed");
        }
        Err(error) if error.tracing_failed() => {
            tracing::debug!(target: "compiler::proof", outcome = "failed", error_kind = error.tracing_kind(), visited_state_count = search.states_visited, complete_assignment_count = search.complete_assignments, feasible_assignment_count = search.feasible_assignments, capability_rejection_count = search.rejected_by_capability, source_rejection_count = search.rejected_by_source, constructibility_rejection_count = search.rejected_by_constructibility, representation_rejection_count = search.rejected_by_representation, disclosure_rejection_count = search.rejected_by_disclosure, message = "proof search completed");
        }
        Err(error) => {
            tracing::debug!(target: "compiler::proof", outcome = "refused", refusal_kind = error.tracing_kind(), visited_state_count = search.states_visited, complete_assignment_count = search.complete_assignments, feasible_assignment_count = search.feasible_assignments, capability_rejection_count = search.rejected_by_capability, source_rejection_count = search.rejected_by_source, constructibility_rejection_count = search.rejected_by_constructibility, representation_rejection_count = search.rejected_by_representation, disclosure_rejection_count = search.rejected_by_disclosure, message = "proof search completed");
        }
    }
    result.map(|candidates| FeasibleProofPlans { candidates, search })
}

#[cfg(test)]
fn enumerate_feasible_plans_core(
    input: &BoundCompilerInput,
    view: &CapabilityView,
    search: &mut ProofSearchReport,
) -> Result<Vec<ProofPlanCandidate>, CompileError> {
    let lowered = crate::engine_attestation::proof::lower(input)?;
    lowered.enumerate(view, input.policy().proof_search_limits, search)
}

/// Local (representation-independent) feasibility of one alternative:
/// capability containment and witness constructibility.
///
/// Returns the alternative's capabilities and source rows when
/// feasible, and otherwise the typed cause of the rejection, so the
/// search records the failure it actually observed.
///
/// # Errors
///
/// The [`LocalProofRejection`] the alternative failed under.
pub fn locally_feasible(
    declaration: &RelationDeclaration,
    alternative: &ProofAlternativeId,
    view: &CapabilityView,
    constructibility: &CompilerConstructibilityAnalysis,
) -> Result<(BTreeSet<RequiredCapability>, Vec<SourceRequirement>), LocalProofRejection> {
    let capabilities = proof_capabilities(declaration, alternative.proof());

    if !view.supports(&capabilities) {
        return Err(LocalProofRejection::MissingCapability);
    }

    let rows = derive_source_requirements(declaration, alternative.proof())
        .map_err(|_| LocalProofRejection::SourceDerivation)?;

    validate_source_constructibility(constructibility, &declaration.id, &rows)
        .map_err(|_| LocalProofRejection::Constructibility)?;

    Ok((capabilities, rows))
}
