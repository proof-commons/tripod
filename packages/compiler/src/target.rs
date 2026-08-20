//! The abstract target requirement boundary (Guide-8 §15, Appendix D).
//!
//! This module is the whole public vocabulary a target adapter needs
//! from the compiler, and deliberately nothing else. It names what an
//! approved analysis requires of *some* target — abstractly. No opcode,
//! leaf version, target capability, evidence identity, transaction
//! position, or target package name appears here or can appear here:
//! the compiler depends on no target package, so a target-specific type
//! is not merely discouraged at this boundary, it is unnameable.
//!
//! # What is exposed, and what is not
//!
//! [`RequiredCapability`] and its complete census, a compiler-owned
//! projection of the external-evidence role, and a read-only
//! [`TargetRequirementSet`]. The analyzed program itself, its proof
//! plans, its placements, its coverage graph, its search reports, and
//! every graph handle inside them stay crate-private. A consumer that
//! genuinely needs one of those will have to justify exposing it on its
//! own terms rather than receive it as a side effect of asking what
//! capabilities an analysis requires.
//!
//! # The evidence role is compiler-owned on purpose
//!
//! The analyzed program carries `realization::ExternalEvidenceRequirement`
//! values, which name an architecture operation and an asset. Exposing
//! that type through this boundary would put a realization type in the
//! adapter's public signature, and the adapter's package contract
//! admits no realization dependency. So the boundary projects the
//! *role* — what class of external claim must be discharged — and drops
//! the operation and asset identities, which are architecture-owned
//! values a target adapter has no business reading. The projection is
//! total and exhaustive: a new realization requirement class fails to
//! compile here until this module states its role.
//!
//! # Union, not intersection
//!
//! Phase 2 selects no proof plan, so a requirement set is the union of
//! what the retained alternatives require, never the intersection. The
//! intersection would be the smaller number and the wrong one: it would
//! silently drop every capability that only one retained alternative
//! needs, which is precisely the weakening a target assessment exists
//! to prevent.

use std::collections::BTreeSet;

use realization::ExternalEvidenceRequirement;

pub use crate::capability::RequiredCapability;
pub use crate::placement::PlacementSearchLimits;
use crate::{
    CompileError,
    analyzed::{ScopedAnalyzedProgram, analyze_scoped_program},
    input::BoundCompilerInput,
};

/// The class of external claim one evidence requirement carries.
///
/// A compiler-owned projection of the realization requirement. It
/// answers "what kind of thing must something outside this analysis
/// establish", and nothing about which operation or asset raised it:
/// those identities belong to the architecture, and a target adapter
/// reading them would be reading a protocol fact through a target
/// interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExternalEvidenceRole {
    /// The substrate itself must conserve value across a transaction.
    ///
    /// No analysis, and no program the analysis could emit, discharges
    /// this. Only the target's own consensus rules do.
    SubstrateConservation,
}

impl ExternalEvidenceRole {
    /// The complete census of evidence roles, in canonical order.
    pub const ALL: &'static [Self] = &[Self::SubstrateConservation];

    /// The role one realization requirement carries.
    ///
    /// Exhaustive with no wildcard arm: a realization requirement class
    /// added later stops this crate compiling until its role is stated,
    /// which is the only mechanism that keeps the census complete.
    pub(crate) const fn of(requirement: &ExternalEvidenceRequirement) -> Self {
        match requirement {
            ExternalEvidenceRequirement::SubstrateConservation { .. } => {
                Self::SubstrateConservation
            }
        }
    }
}

/// What one validated analysis requires of a target.
///
/// Read-only and opaque. The fields are private and there is no public
/// constructor: the only way to obtain one is
/// [`analyze_target_requirements`], which runs the complete analysis
/// and its independent validator first. A set assembled from arbitrary
/// capabilities would be a request rather than an analysis, and nothing
/// downstream could tell the two apart once they shared a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetRequirementSet {
    capabilities: BTreeSet<RequiredCapability>,
    external_evidence: BTreeSet<ExternalEvidenceRole>,
}

impl TargetRequirementSet {
    /// Every abstract capability some retained alternative requires, in
    /// canonical census order.
    pub fn capabilities(&self) -> impl Iterator<Item = RequiredCapability> + '_ {
        self.capabilities.iter().copied()
    }

    /// Every external-evidence role the analysis leaves open, in
    /// canonical census order.
    pub fn external_evidence(&self) -> impl Iterator<Item = ExternalEvidenceRole> + '_ {
        self.external_evidence.iter().copied()
    }
}

/// Analyze one bound input into its abstract target requirements.
///
/// The five steps of Guide-8 §15.5 in order: the scoped analyzed
/// program is assembled, the complete re-derivation validator runs
/// inside that assembly, the derived censuses are checked for
/// duplicates and canonical order, the requirement set is derived, and
/// an opaque read-only projection is returned. A caller receives the
/// requirements or a typed failure; there is no partial result and no
/// route to the analyzed program itself.
///
/// # Errors
///
/// Any failure of the scoped analysis, including its complete
/// validator; a typed census defect
/// ([`CompileError::NoncanonicalCapabilityCensus`] or
/// [`CompileError::NoncanonicalEvidenceRoleCensus`]) or a capability
/// closure mismatch
/// ([`CompileError::AnalyzedCapabilityClosureMismatch`]) raised while
/// projecting the requirements.
pub fn analyze_target_requirements(
    input: &BoundCompilerInput,
    placement_limits: PlacementSearchLimits,
) -> Result<TargetRequirementSet, CompileError> {
    let analyzed = analyze_scoped_program(input, placement_limits)?;

    project_target_requirements(&analyzed)
}

/// Derive the requirement set from a complete analyzed program.
///
/// Crate-private, and reachable only below [`analyze_target_requirements`],
/// so the validated-analysis precondition is a property of the call
/// graph rather than a comment. The plan aggregates are re-checked
/// against their relation-owned unions here rather than trusted:
/// assembly already checks that closure, and a projection that trusted
/// it would be the one place where a corrupted aggregate reached a
/// public boundary intact.
///
/// # Errors
///
/// [`CompileError::AnalyzedCapabilityClosureMismatch`] when a plan's
/// capability aggregate is not exactly the union its relations own;
/// [`CompileError::NoncanonicalCapabilityCensus`] or
/// [`CompileError::NoncanonicalEvidenceRoleCensus`] when a derived
/// census repeats a member, orders it noncanonically, or names a member
/// the census constant omits.
pub(crate) fn project_target_requirements(
    analyzed: &ScopedAnalyzedProgram,
) -> Result<TargetRequirementSet, CompileError> {
    let mut capabilities = BTreeSet::new();

    for plan in analyzed.proof_plans.values() {
        let mut owned = BTreeSet::new();

        for (relation, bundle) in &plan.relation_requirements {
            for capability in &bundle.required_capabilities {
                owned.insert(*capability);

                if !plan.proof_plan.required_capabilities.contains(capability) {
                    return Err(CompileError::AnalyzedCapabilityClosureMismatch {
                        relation: Some(relation.clone()),
                        capability: *capability,
                    });
                }
            }
        }

        if let Some(capability) = plan
            .proof_plan
            .required_capabilities
            .difference(&owned)
            .next()
        {
            return Err(CompileError::AnalyzedCapabilityClosureMismatch {
                relation: None,
                capability: *capability,
            });
        }

        capabilities.extend(owned);
    }

    let evidence = analyzed
        .required_external_evidence
        .iter()
        .map(ExternalEvidenceRole::of)
        .collect::<BTreeSet<_>>();

    let capabilities = canonical_census(&capabilities, RequiredCapability::ALL, |capability| {
        CompileError::NoncanonicalCapabilityCensus { capability }
    })?;
    let external_evidence = canonical_census(&evidence, ExternalEvidenceRole::ALL, |role| {
        CompileError::NoncanonicalEvidenceRoleCensus { role }
    })?;

    Ok(TargetRequirementSet {
        capabilities: capabilities.into_iter().collect(),
        external_evidence: external_evidence.into_iter().collect(),
    })
}

/// Order the present members by the census constant, exactly.
///
/// The constant is the contract: its order must be the type's own
/// canonical order and it must name every member exactly once. Checking
/// the vector rather than a set is the whole point — a set comparison
/// accepts a census that lists one member twice, and a census that
/// disagrees with itself about how many members it has is not a census.
///
/// Returns the present members in census order.
pub(crate) fn canonical_census<T: Copy + Ord>(
    present: &BTreeSet<T>,
    census: &[T],
    error: impl Fn(T) -> CompileError,
) -> Result<Vec<T>, CompileError> {
    if let Some(pair) = census.windows(2).find(|pair| pair[0] >= pair[1]) {
        return Err(error(pair[1]));
    }

    if let Some(unknown) = present.iter().find(|member| !census.contains(member)) {
        return Err(error(*unknown));
    }

    Ok(census
        .iter()
        .copied()
        .filter(|member| present.contains(member))
        .collect())
}
