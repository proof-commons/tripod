//! Default-library fixed-form keyed plans. Runtime obligations are symbolic: no transaction,
//! evidence witness or target execution is manufactured by this interface.
use crate::input::{BoundCompilerInput, KeyedFixedOwner};
use architecture::keyed_fixed::KeyedFixedArchitectureBinding;
use architecture::keyed_fixed::{KeyedFixedObjectId, KeyedFixedOperationId};
use realization::keyed_fixed_relation::{KeyedFixedRelation, KeyedFixedRelationId};
use std::collections::{BTreeMap, BTreeSet};

pub use crate::carrier::CarrierQuantification;
pub use crate::coverage::CoverageBoundary;
pub use crate::coverage_graph::{CoverageEdge, CoverageEdgeClass, CoverageNodeClass};
pub use crate::engine_keyed_fixed::ShellField as DeclarationField;
pub use crate::input::ProofSearchLimits;
pub use crate::placement::{CarrierMultiplicity, PlacementSearchLimits, PlacementSearchReport};
pub use crate::proof::{LocalProofRejection, ProofSearchReport};

/// A premise still owed at the evidence boundary, not an admitted witness.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EvidenceObligation {
    pub relation: KeyedFixedRelationId,
    pub requirement: KeyedFixedRelation,
    pub premises: EvidencePremises,
}
/// Explicit assumptions consumed by recognition and membership reading.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidencePremises {
    HandleClassification {
        side: realization::TransactionSide,
        minimum: u16,
        maximum: Option<u16>,
    },
    SourceClassification,
    MembershipReading {
        recurrence: realization::keyed_fixed_relation::RecordRecurrence,
        inclusion: bool,
        retrieval: bool,
        finality_for_final_reading: bool,
    },
}
/// One runtime check universally quantified over the consumed handle family.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeCarrier {
    pub operation: KeyedFixedOperationId,
    pub object: KeyedFixedObjectId,
    pub quantification: CarrierQuantification,
}
/// The evidence defect a negative coverage subject must demonstrate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceDefect {
    Missing,
    Failed,
    IdentityMismatch,
}
/// Owner-typed coverage purpose, including the shared attachment obligations.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoveragePurpose {
    Verified(EvidenceObligation),
    Rejected {
        evidence: EvidenceObligation,
        defect: EvidenceDefect,
    },
    ClauseAccepted,
    ClauseRejected,
    AcceptedProjection,
    CarrierExecution(RuntimeCarrier),
}
/// A positive or negative requirement at its declared boundary.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoverageRequirement {
    pub relation: KeyedFixedRelationId,
    pub boundary: CoverageBoundary,
    pub purpose: CoveragePurpose,
    pub inactive: bool,
}
/// Runtime clause restored from the shared case facts, not inferred from a carrier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeClause {
    pub side: realization::TransactionSide,
    pub multiplicity: CarrierMultiplicity,
}
/// Complete per-relation publication from a shared operation bundle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationPlan {
    pub declaration: KeyedFixedRelation,
    pub active: bool,
    pub runtime: Vec<RuntimeClause>,
    pub boundaries: BTreeSet<CoverageBoundary>,
    pub positive: Vec<CoverageRequirement>,
    pub negative: Vec<CoverageRequirement>,
    pub evidence: BTreeSet<EvidenceObligation>,
    pub alternatives: BTreeSet<BTreeSet<RuntimeCarrier>>,
    pub projections: BTreeSet<CoverageBoundary>,
}
/// Complete owner-typed coverage graph symbol.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoverageSymbol {
    Relation(KeyedFixedRelationId),
    Requirement(Box<CoverageRequirement>),
    Evidence(EvidenceObligation),
    Carrier {
        relation: KeyedFixedRelationId,
        carrier: RuntimeCarrier,
    },
}
/// A node from the shared coverage graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverageNode {
    pub id: CoverageSymbol,
    pub operation: KeyedFixedOperationId,
    pub class: CoverageNodeClass,
    pub relation: Option<KeyedFixedRelationId>,
    pub active: Option<bool>,
}
/// A typed edge from the shared coverage graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverageDependency {
    pub source: CoverageSymbol,
    pub target: CoverageSymbol,
    pub edge: CoverageEdge,
}
/// One operation factor; alternatives across operations remain factorized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationPlan {
    pub relations: BTreeMap<KeyedFixedRelationId, RelationPlan>,
    pub placements: Vec<BTreeMap<KeyedFixedRelationId, Vec<RuntimeCarrier>>>,
    pub coverage_nodes: Vec<CoverageNode>,
    pub coverage_edges: Vec<CoverageDependency>,
    pub search: PlacementSearchReport,
}
/// One retained proof candidate, including relation-owned evidence requirements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofPlan {
    pub selected_relations: BTreeSet<KeyedFixedRelationId>,
    pub requirements: BTreeMap<KeyedFixedRelationId, BTreeSet<EvidenceObligation>>,
    pub operations: BTreeMap<KeyedFixedOperationId, OperationPlan>,
}
/// Observed stage censuses; zero means the declaration-derived stage was empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageCensus {
    pub relations: usize,
    pub source_tables: usize,
    pub construction_nodes: usize,
    pub lifecycle_nodes: usize,
    pub proof_plans: usize,
    pub placement_assignments: usize,
    pub coverage_nodes: usize,
}
/// Readable publication. Mutating a clone cannot mutate an immutable `KeyedFixedPlan`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedPlanProjection {
    pub binding: KeyedFixedArchitectureBinding,
    pub scope: Vec<KeyedFixedOperationId>,
    pub plans: Vec<ProofPlan>,
    pub evidence: BTreeSet<EvidenceObligation>,
    pub stages: StageCensus,
    pub proof_limits: ProofSearchLimits,
    pub placement_limits: PlacementSearchLimits,
    pub proof_search: ProofSearchReport,
}
/// Immutable validated plan, constructed only by [`plan`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedPlan {
    pub(crate) projection: KeyedFixedPlanProjection,
}
impl KeyedFixedPlan {
    /// The complete typed publication; no engine record is exposed.
    #[must_use]
    pub const fn project(&self) -> &KeyedFixedPlanProjection {
        &self.projection
    }
}
/// Stage at which a keyed declaration or its lowering was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Stage {
    Lowering,
    Relations,
    Predicates,
    Sources,
    Constructibility,
    Lifecycle,
    Proof,
    Requirements,
    Cases,
    Placement,
    Coverage,
    Join,
}
/// Typed record defect. These are internal contract failures, not partial plans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Defect {
    MissingSourceTable,
    RuntimeSourcePayload,
    ForeignProofOwner,
    DuplicateRelation,
    UnknownEndpoint,
    EscapesScope,
    DuplicateDependency,
    Cycle,
    CensusMismatch,
    DuplicateOperand,
    MissingOperand,
    ForeignOperand,
    DuplicateRow,
    /// Partial restoration fallback; complete identities use the dedicated error.
    UndecidableRegion,
    NoFeasiblePlan,
    FixedSource,
    ForeignEvidence,
    UnsupportedRuntimeScope,
    UnsupportedEvidenceSide,
    RuntimeObjectMismatch,
    UnexpectedLayoutRequirement,
    RepresentationChoices,
    LifecycleRequirements,
    Constructibility,
    SourceRows,
    DuplicateSourceTable,
    UnnamedSourceTable,
    ProofSource,
    /// Only the private raw-record test oracle can return a lowering wrapper.
    #[cfg(test)]
    InvalidRecords,
    UnknownExpression,
    NonBoolean,
    OutsideScope,
    DuplicateConstructibilityNode,
    DuplicateConstructibilityDependency,
    UnknownConstructibilityNode,
    ConstructibilityCycle,
    ConstructibilityCrossOperationDependency,
    PermissionlessPrivateDependency,
    ConstructibilityWitnessUnavailable,
    SponsorDependencyEscaped,
    MissingRepresentationChoice,
    MissingLifecyclePath,
    UnsupportedRepresentation,
    MissingSelection,
    UnexpectedSelection,
    UnapprovedSelection,
    DuplicateSelection,
    CapabilityMismatch,
    SourceMismatch,
    EvidenceMismatch,
    DuplicateCase,
    CaseFactsMismatch,
    NoEligibleCarrier,
    GlobalRelationHasOnlyLocalCarrier,
    UnconditionalRelationOnOptionalCarrier,
    NonCanonicalCarrierAssignment,
    MissingLayoutRequirement,
    DuplicatePlacedCarrier,
    UnpermittedCarrierPlacement,
    DuplicateRequirement,
    MissingInactive,
    UnexpectedRejection,
    MissingPositive,
    MissingNegative,
    UnexpectedCarrier,
    MissingCarrier,
    MissingProjection,
    UnexpectedLayout,
    MissingLayout,
    UnknownSymbol,
    DuplicateSymbol,
    EndpointClass,
    CrossOperation,
    Prerequisite,
}
/// Public refusals contain owner identities, never shared engine records.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum KeyedFixedPlanError {
    #[error("fixed-form keyed proof bundle refused for {relation:?}: {reason:?}")]
    ProofBundle {
        relation: KeyedFixedRelationId,
        reason: Option<LocalProofRejection>,
    },
    #[error("fixed-form keyed coverage graph refused: {defect:?} at {symbols:?}")]
    CoverageGraph {
        defect: Defect,
        symbols: Vec<CoverageSymbol>,
        unrestored_key_count: usize,
    },
    #[error("unsupported fixed-form keyed declaration field {field:?} in {operation:?}: {count}")]
    UnsupportedShell {
        operation: KeyedFixedOperationId,
        field: DeclarationField,
        count: usize,
    },
    #[error("undecidable fixed-form keyed region for {0:?}")]
    UndecidableRegion(KeyedFixedOperationId),
    #[error("fixed-form keyed {stage:?} refused: {defect:?} at {relations:?}")]
    /// Census subjects merge missing and unexpected identities into a sorted,
    /// deduplicated union. The count records unrestored key occurrences.
    Refused {
        stage: Stage,
        defect: Defect,
        relations: Vec<KeyedFixedRelationId>,
        operations: Vec<KeyedFixedOperationId>,
        unrestored_key_count: usize,
    },
    #[error("fixed-form keyed proof source is unavailable for {0:?}")]
    FixedSource(KeyedFixedRelationId),
    #[error("no feasible fixed-form keyed proof plan for {0:?}")]
    NoFeasiblePlan(Vec<KeyedFixedRelationId>),
    #[error("fixed-form keyed {stage:?} search limit {limit} exceeded (candidates: {candidates})")]
    SearchLimit {
        stage: Stage,
        candidates: bool,
        limit: u64,
    },
    #[error("fixed-form keyed publication does not equal fresh bound-source derivation")]
    PublicationMismatch,
    #[error(transparent)]
    Compiler(#[from] Box<crate::CompileError>),
}

/// Compile all feasible keyed plans and independently revalidate publication.
/// # Errors
/// Returns a typed lowering, shared-stage or publication refusal, never a partial plan.
pub fn plan(
    input: &BoundCompilerInput<KeyedFixedOwner>,
    limits: PlacementSearchLimits,
) -> Result<KeyedFixedPlan, KeyedFixedPlanError> {
    let span = tracing::debug_span!(target: "compiler::keyed_fixed_plan", "plan", scope_count = crate::instrumentation::count(input.owner().scope().len()));
    let _entered = span.enter();
    let result = crate::engine_keyed_fixed::program::derive(input, limits).and_then(|projection| {
        let plan = KeyedFixedPlan { projection };
        revalidate_core(input, limits, &plan)?;
        Ok(plan)
    });
    tracing::debug!(target: "compiler::keyed_fixed_plan", outcome = if result.is_ok() { "completed" } else { "refused" }, "keyed plan completed");
    result
}
/// Rerun fresh preparation and every shared stage from the bound source.
/// No producer preparation or raw record cache is accepted by this interface.
/// # Errors
/// Returns the fresh derivation's typed error, or `PublicationMismatch`.
pub fn revalidate(
    input: &BoundCompilerInput<KeyedFixedOwner>,
    limits: PlacementSearchLimits,
    offered: &KeyedFixedPlan,
) -> Result<(), KeyedFixedPlanError> {
    let span = tracing::debug_span!(target: "compiler::keyed_fixed_plan", "revalidate");
    let _entered = span.enter();
    let result = revalidate_core(input, limits, offered);
    tracing::debug!(target: "compiler::keyed_fixed_plan", outcome = if result.is_ok() { "completed" } else { "refused" }, "keyed plan revalidated");
    result
}
fn revalidate_core(
    input: &BoundCompilerInput<KeyedFixedOwner>,
    limits: PlacementSearchLimits,
    offered: &KeyedFixedPlan,
) -> Result<(), KeyedFixedPlanError> {
    let expected = crate::engine_keyed_fixed::program::derive(input, limits)?;
    if expected != offered.projection {
        return Err(KeyedFixedPlanError::PublicationMismatch);
    }
    Ok(())
}
