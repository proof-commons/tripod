//! Typed case facts retained before any proof or placement selection.
use crate::{
    coverage::CoverageBoundary,
    engine_coverage_join::Alternative,
    engine_lifecycle::{ModeKey, ObjectKey},
    engine_placement::{Key, LayoutKey},
    engine_records::OperationKey,
    engine_source::{Activation, Evidence, SourceRow},
};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Constructibility {
    PublicPermissionless,
    Operator,
    OwnersOf(ObjectKey),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompilerRequirement {
    Representation {
        object: ObjectKey,
        allowed: BTreeSet<ModeKey>,
        selected: ModeKey,
    },
    Constructibility(Constructibility),
    Lifecycle {
        object: ObjectKey,
        exit: OperationKey,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructuralRequirement {
    Representation {
        object: ObjectKey,
        mode: ModeKey,
    },
    SecretFree(OperationKey),
    Lifecycle {
        object: ObjectKey,
        exit: OperationKey,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    Member { side: u16, object: ObjectKey },
    Family { side: u16, object: ObjectKey },
    Side(u16),
    Object(ObjectKey),
    Transaction,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Runtime {
    pub scope: Scope,
    pub multiplicity: crate::placement::CarrierMultiplicity,
    pub sources: Vec<SourceRow>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    pub key: Key,
    pub case_operation: OperationKey,
    pub activation: Activation,
    pub active: bool,
    pub boundaries: BTreeSet<CoverageBoundary>,
    pub sources: BTreeSet<SourceRow>,
    pub compiler: BTreeSet<CompilerRequirement>,
    pub structural: BTreeSet<StructuralRequirement>,
    pub runtime: Vec<Runtime>,
    pub evidence: BTreeSet<Evidence>,
    pub layout: BTreeSet<LayoutKey>,
}
pub struct Bundle {
    pub facts: Facts,
    pub carriers: BTreeSet<Alternative>,
    pub coverage: crate::engine_coverage_join::Joined,
}
pub fn validate(
    operation: OperationKey,
    census: &BTreeSet<LayoutKey>,
    bundle: &Bundle,
) -> Result<(), crate::analyzed_operation::RelationCaseDefect> {
    use crate::analyzed_operation::RelationCaseDefect as D;
    let span = tracing::debug_span!(target: "compiler::engine_case_facts", "validate", source_count = crate::instrumentation::count(bundle.facts.sources.len()), carrier_count = crate::instrumentation::count(bundle.carriers.len()));
    let _entered = span.enter();
    let facts = &bundle.facts;
    if facts.key.relation.operation != operation || facts.case_operation != operation {
        return Err(D::ForeignOperation);
    }
    if facts.active != bundle.coverage.plan.active {
        return Err(D::ActivityMismatch);
    }
    if facts.boundaries != bundle.coverage.plan.boundaries {
        return Err(D::BoundaryMismatch);
    }
    if bundle.carriers != bundle.coverage.carrier.clone().unwrap_or_default() {
        return Err(D::CarrierMismatch);
    }
    if facts.evidence != bundle.coverage.plan.evidence {
        return Err(D::EvidenceMismatch);
    }
    let runtime = facts.boundaries.contains(&CoverageBoundary::RuntimeCarrier);
    if !facts.active && !facts.sources.is_empty() {
        return Err(D::ActiveSourceOnInactiveCase);
    }
    if (!facts.active || !runtime) && !bundle.carriers.is_empty() {
        return Err(D::CarrierOnNonRuntimeCase);
    }
    if runtime && facts.active && bundle.carriers.is_empty() {
        return Err(D::MissingRuntimeCarrier);
    }
    if facts.boundaries.contains(&CoverageBoundary::CompilerStatic) == facts.compiler.is_empty() {
        return Err(D::CompilerRequirementMismatch);
    }
    if facts
        .boundaries
        .contains(&CoverageBoundary::BackendStructural)
        == facts.structural.is_empty()
    {
        return Err(D::StructuralRequirementMismatch);
    }
    if facts
        .boundaries
        .contains(&CoverageBoundary::ExternalEvidence)
        == facts.evidence.is_empty()
    {
        return Err(D::ExternalEvidenceMismatch);
    }
    if !facts.layout.is_subset(census) {
        return Err(D::SurplusLayoutRequirement);
    }
    Ok(())
}
