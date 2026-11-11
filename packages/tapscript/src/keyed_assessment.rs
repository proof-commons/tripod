//! Keyed emission's static target obligations, without ASH recognition rules.
//!
//! A reviewed capability is not activation or transaction admission. The complete
//! version/program comparisons are already in the retained continuation fragment;
//! this assessment names their target primitives and preserves all evidence rows.
use crate::{
    TapscriptError,
    keyed_bundle::KeyedEmissionBundle,
    keyed_constructor::KeyedLeafRecipe,
    keyed_resources::{KeyedResourceProjection, KeyedSchedule, project},
};
use compiler::keyed_plan::EvidenceObligation;
use std::collections::{BTreeMap, BTreeSet};
use target_elements::{
    CapabilityContract, ElementsCapability, ReviewedElementsTapscriptDefinition,
    StaticCapabilityStatus, ValidatedTargetDefinition,
};

/// Static reviewed requirements, not a deployment or predecessor certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct KeyedTargetAssessment {
    capabilities: BTreeMap<ElementsCapability, CapabilityContract>,
    recipe: KeyedLeafRecipe,
    resources: KeyedResourceProjection,
    evidence: BTreeSet<EvidenceObligation>,
}

impl KeyedTargetAssessment {
    #[must_use]
    pub const fn capabilities(&self) -> &BTreeMap<ElementsCapability, CapabilityContract> {
        &self.capabilities
    }
    #[must_use]
    pub const fn recipe(&self) -> &KeyedLeafRecipe {
        &self.recipe
    }
    #[must_use]
    pub const fn resources(&self) -> &KeyedResourceProjection {
        &self.resources
    }
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<EvidenceObligation> {
        &self.evidence
    }
}

/// Static assessment failures preserve the target capability identity.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedAssessmentRefusal {
    /// None means absent; Some preserves Unsupported versus Incomplete.
    Capability {
        capability: ElementsCapability,
        status: Option<StaticCapabilityStatus>,
    },
    Resources(TapscriptError),
}

impl KeyedAssessmentRefusal {
    /// Stable payload-free tracing classification.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match self {
            Self::Capability { .. } => "capability",
            Self::Resources(_) => "resources",
        }
    }
}

/// Assess precisely the keyed grammar against the reviewed target contracts.
///
/// Outpoint authentication means the target's spent-output introspection context,
/// not production verification of a predecessor control block. The latter remains
/// an explicit offline input at member binding. Concrete deployment, package,
/// funding and whole-transaction resource obligations remain unresolved.
/// The resource core uses the same fixed grammar with key zero: every key is
/// exactly a 32-byte literal consumed by DROP before either clause. Changing its
/// bytes changes neither instruction widths, stack schedule nor resource usage.
/// Complete version/payload equality is pinned by the continuation grammar, not
/// restated as a mutable Boolean on this assessment.
/// # Errors
/// Refuses an unavailable reviewed primitive or a resource projection failure.
pub fn assess_keyed_bundle(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &KeyedEmissionBundle,
) -> Result<KeyedTargetAssessment, KeyedAssessmentRefusal> {
    let _span = tracing::debug_span!(target: "tapscript::keyed_assessment", "assess_keyed_bundle")
        .entered();
    let result = assess(target, bundle);
    match &result {
        Ok(_) => {
            tracing::debug!(target: "tapscript::keyed_assessment", outcome = "completed", "keyed target assessed");
        }
        Err(error) => {
            tracing::debug!(target: "tapscript::keyed_assessment", outcome = "refused", refusal_kind = error.refusal_kind(), "keyed target assessed");
        }
    }
    result
}

fn assess(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &KeyedEmissionBundle,
) -> Result<KeyedTargetAssessment, KeyedAssessmentRefusal> {
    let capabilities =
        assess_capabilities(target.validated(), bundle.project().successor.program())?;
    Ok(KeyedTargetAssessment {
        capabilities,
        recipe: *bundle.project().successor.recipe(),
        resources: project(target, KeyedSchedule::Leaf)
            .map_err(KeyedAssessmentRefusal::Resources)?,
        evidence: bundle.project().evidence.clone(),
    })
}

// The validated core permits non-weakening controls without exposing a public
// route that could impersonate the reviewed target. Absent registry rows cannot
// pass target validation; status downgrades are the reachable negative subjects.
pub(crate) fn assess_capabilities(
    target: &ValidatedTargetDefinition,
    program: &crate::TapscriptProgram,
) -> Result<BTreeMap<ElementsCapability, CapabilityContract>, KeyedAssessmentRefusal> {
    let required = crate::pattern::fragment_prerequisites(program)
        .into_iter()
        .chain([
            ElementsCapability::TapscriptExecution,
            ElementsCapability::RequiredLeafVersion,
        ]);
    let contracts = target.definition().capabilities();
    required
        .map(|capability| {
            let contract = contracts.get(&capability);
            let status = contract.map(CapabilityContract::status);
            match contract {
                Some(row) if status == Some(StaticCapabilityStatus::Reviewed) => {
                    Ok((capability, row.clone()))
                }
                _ => Err(KeyedAssessmentRefusal::Capability { capability, status }),
            }
        })
        .collect()
}
