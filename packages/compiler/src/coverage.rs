//! Relation-indexed coverage requirements (Guide-6 Tranches B–D, H).
//!
//! Coverage says "evidence is required"; it never says "evidence has
//! been produced". Every requirement here is an obligation recorded
//! against a future evidence package, and no requirement carries a
//! mutable completion status, a report, a verdict, or a fixture.
//!
//! Coverage identity is the relation-case pair, and every in-scope
//! relation appears in every applicable execution case exactly once —
//! including when its disposition is compiler-static, backend-
//! structural, external evidence, or inactive. A relation must never
//! disappear from the census because no runtime carrier executes it:
//! inactivity is a stated disposition, and an omission is
//! indistinguishable from a relation nobody analyzed.
//!
//! The evidence boundary is load-bearing rather than decorative. A
//! compiler-static relation's positive coverage is "valid typed input
//! accepted" and its negative is "malformed typed input rejected"; a
//! backend-structural relation's is "the required structure is present"
//! and "missing or contradictory structure rejects"; only a runtime
//! relation's is "the target accepts the valid operation and rejects a
//! focused mutation". Forcing every relation kind into the runtime pair
//! would claim target execution for obligations no target ever
//! evaluates, so the boundary travels in the requirement identity and
//! the derivation branches on it.
//!
//! The mutation catalogue is an exhaustive match over the realization
//! relation enum with no wildcard arm, so a new relation variant fails
//! to compile here until its required negative classes are stated. A
//! mutation is a semantic class, not a mutated world: this stage
//! mutates nothing and constructs no target transaction.

// The item-level allowances below are the scope-wide coverage stack
// and its projections. The canonical path analyzes coverage per
// operation (§10.1) and never aggregates across the scope, so the
// scope-wide analysis is retained for the independent oracles and the
// phase-exit product comparison rather than called in production.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{BoundId, OperationId};
use realization::{
    CardinalityMaximum, ConstructibilityClass, Count, ExternalEvidenceRequirement, Relation,
    RelationDeclaration, RelationId, RepresentationMode,
};

use crate::{
    CompileError,
    capability::RequiredCapability,
    carrier::{CarrierEligibility, relation_case_eligibility},
    case::ExecutionCaseId,
    coverage_graph::{CoverageGraphProjection, resolve_coverage_dependencies},
    layout::{LayoutRequirement, names_sponsor_amount, selected_carrier_requirements},
    placement::{
        PlacedCarrier, PlacedProofPlanCandidate, PlacedProofPlans, PlacementCandidate,
        RelationActivity, RelationCaseKey, RelationCasePlan,
    },
    proof::ProofPlanCandidate,
    relation::CompilerRelationAnalysis,
    source::{OperandId, SourceRequirement, is_sponsor_amount_operand, relation_operands},
};

/// Where one coverage requirement is answered.
///
/// The Guide-5 discharge boundary is reused rather than restated: a
/// second enum with the same four members would be a second source of
/// truth that can drift from the classification coverage consumes. The
/// fifth row of the Guide-6 boundary table — the inactive case — is a
/// disposition rather than a boundary, and is carried by
/// [`RelationActivity`] instead.
pub use crate::placement::DischargeBoundary as CoverageBoundary;

/// The declared ceiling one above-maximum mutation must exceed.
///
/// An architecture-owned bound is cited, never copied: the later vector
/// materializer reads the bound's exact value and adds one, and a value
/// copied into a compiler requirement now would be a draft that can
/// drift from the architecture that owns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CardinalityCeiling {
    /// The relation declares an exact maximum.
    Declared(Count),
    /// The relation's maximum is owned by an architecture bound.
    Bound(BoundId),
}

/// One required negative coverage class.
///
/// A semantic class, never a concrete malformed transaction: this stage
/// names what a later evidence package must make invalid, and nothing
/// here can be executed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationMutation {
    CardinalityBelowMinimum,
    CardinalityAboveMaximum { ceiling: CardinalityCeiling },
    UndeclaredObjectFamily,
    WrongRecognizedAsset,
    WrongRecognizedObject,
    AmountMismatch,
    MissingRequiredOwner,
    UnexpectedProtocolSecret,
    SponsorProtocolOverlap,
    MissingSponsorAuthorization,
    SponsorEnvelopeMultiplicityExceeded,
    WrongRootEffect,
    MissingRequiredProjection,
    ForbiddenProjectionPresent,
    MissingCanonicalDeltaFamily,
    UnexpectedCanonicalDeltaFamily,
    DuplicateCanonicalSourceOrDestination,
    UndeclaredOpenFlow,
    ConstructibilityWitnessUnavailable,
    PermissionlessPrivateDependency,
    UnsupportedRepresentation,
    UnauthenticatedRepresentation,
    RequiredLifecycleExitMissing,
    ExpressionPredicateFalse,
    ExternalEvidenceMissing,
    ExternalEvidenceFailed,
    ExternalEvidenceIdentityMismatch,
}

/// What one coverage requirement asks a later evidence package for.
///
/// Equivalent factoring of the Guide-6 purpose vocabulary. The
/// boundary is not folded in here — it is a separate component of the
/// requirement identity — so one hybrid relation can state a
/// compiler-static acceptance and a backend-structural presence
/// without the two colliding.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoveragePurpose {
    /// The boundary accepts the valid subject.
    ActiveAccept,
    /// The case is valid and the relation is not activated.
    InactiveAccept,
    /// One focused mutation is rejected.
    FocusedReject(RelationMutation),
    /// One selected carrier executes the relation.
    CarrierExecution(PlacedCarrier),
    /// The accepted semantic projection is compared, not only the
    /// verdict.
    AcceptedProjection,
    /// A required structural property is present.
    StructuralPresence,
    /// A missing or contradictory structural property is rejected.
    StructuralRejection(RelationMutation),
    /// A typed external report verifies the exact subject.
    ExternalEvidenceVerified {
        requirement: ExternalEvidenceRequirement,
    },
    /// A missing, failed, stale, mismatched, or malformed external
    /// report is rejected.
    ExternalEvidenceRejected {
        requirement: ExternalEvidenceRequirement,
        mutation: RelationMutation,
    },
}

/// Stable identity of one coverage requirement.
///
/// Typed semantic components only: no digest, no graph handle, no
/// candidate vector index, no target program identity, and no target
/// position.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoverageRequirementId {
    pub relation: RelationId,
    pub case: ExecutionCaseId,
    pub boundary: CoverageBoundary,
    pub purpose: CoveragePurpose,
}

impl CoverageRequirementId {
    /// This requirement's relation-case census key.
    #[must_use]
    pub fn key(&self) -> RelationCaseKey {
        RelationCaseKey {
            relation: self.relation.clone(),
            case: self.case.clone(),
        }
    }
}

/// Who answers one coverage requirement.
///
/// The role is not the boundary restated: it names the artifact that
/// carries the answer, so a later package cannot satisfy a structural
/// obligation with a target execution report or the reverse.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceRole {
    /// The compiler's own typed analysis result.
    CompilerAnalysisResult,
    /// A structural fact of the emitted bundle or ABI.
    EmittedStructure,
    /// Execution of the operation at a target.
    TargetExecution,
    /// A typed external report, with the capability it depends on.
    ///
    /// The capability is retained and the evidence stays unresolved:
    /// no runtime carrier and no layout obligation may stand in for a
    /// report that does not exist yet.
    ExternalReport {
        requirement: ExternalEvidenceRequirement,
        capability: RequiredCapability,
    },
    /// Acceptance of a valid case in which the relation never
    /// activates.
    InactiveCaseAcceptance,
}

/// What a later report must claim about dependent relations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollateralPolicy {
    /// The intended relation and the typed dependency closure must
    /// both be reported blocked.
    RequireIntendedAndDependencyClosure,
    /// Only the intended relation is required; anything further is
    /// reported rather than claimed in advance.
    ReportAdditional,
}

/// The dependency collateral one negative requirement expects.
///
/// The closure itself is derived from the coverage dependency graph,
/// which is a later stage: this value carries the policy and an empty
/// closure until [`bind_dependency_collateral`] fills it. An empty
/// closure is therefore "not yet derived", never "proved independent",
/// and no requirement here claims a mutation is isolated.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CollateralRequirement {
    pub policy: CollateralPolicy,
    /// The active dependents that must become blocked, once derived.
    pub dependency_closure: BTreeSet<RelationCaseKey>,
}

/// One required positive coverage obligation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PositiveCoverageRequirement {
    pub id: CoverageRequirementId,
    pub role: EvidenceRole,
    /// The representation the case fixed for this relation's object,
    /// where the relation names one.
    pub representation: Option<RepresentationMode>,
    /// The semantic operands a later report compares, canonically
    /// sorted.
    pub operands: Vec<OperandId>,
}

/// One required negative coverage obligation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NegativeCoverageRequirement {
    pub id: CoverageRequirementId,
    pub role: EvidenceRole,
    pub mutation: RelationMutation,
    pub collateral: CollateralRequirement,
}

/// One carrier assignment a later target plan may select.
///
/// The layout dependencies travel with the alternative rather than with
/// the relation: coverage requires what the *selected* assignment
/// depends on, not the union over every eligible carrier, and a plan
/// that selects a coordinator must not inherit the obligations of the
/// per-member alternative it declined.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CarrierAssignmentAlternative {
    /// Non-empty; the multiplicity travels inside each placed carrier.
    pub carriers: BTreeSet<PlacedCarrier>,
    pub layout: BTreeSet<LayoutRequirement>,
}

/// The carrier obligation of one active runtime relation-case.
///
/// Deliberately not bound to the combined placement product: the
/// alternatives are compressed per relation-case, so a plan with
/// hundreds of feasible whole-transaction placements still states the
/// handful of assignments this one relation actually admits. A later
/// target plan selects one alternative; every carrier in it must be
/// reachable and execute; no carrier outside any alternative may claim
/// discharge; and no evidence is required for the alternatives that
/// were not selected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CarrierCoverageRequirement {
    pub relation_case: RelationCaseKey,
    /// Canonical, deduplicated, and non-empty.
    pub allowed_assignments: BTreeSet<CarrierAssignmentAlternative>,
}

/// What one accepted-projection requirement compares.
///
/// Not every boundary compares a target execution: a compiler-static
/// relation compares the compiler's own selection or validation result,
/// a structural relation compares a fact of the emitted bundle, and an
/// external relation compares an evidence role and subject. Calling any
/// of those "target execution" would claim evidence nobody produced.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectionSubject {
    /// The relation's runtime verdict and its public operands.
    RuntimeRelationVerdict,
    /// The compiler's selection or validation result.
    CompilerSelectionResult,
    /// One structural fact of the emitted bundle or ABI.
    EmittedStructuralFact,
    /// The evidence role and exact subjects of the external reports.
    ExternalReportSubjects {
        requirements: BTreeSet<ExternalEvidenceRequirement>,
    },
}

/// The accepted semantic projection one boundary must compare.
///
/// Facts, not only verdicts: a report that says "accepted" without
/// naming what was accepted cannot distinguish a relation that held
/// from one that was never evaluated.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemanticProjectionRequirement {
    pub relation: RelationId,
    pub case: ExecutionCaseId,
    pub boundary: CoverageBoundary,
    pub subject: ProjectionSubject,
    /// Canonically sorted; never an erased sponsor amount.
    pub operands: Vec<OperandId>,
    /// The active source rows the comparison covers.
    pub sources: Vec<SourceRequirement>,
    pub compare_relation_verdict: bool,
}

/// One relation's complete coverage contract in one execution case.
///
/// Hybrid relations are first class: a representation relation states a
/// compiler-static selection requirement and a backend-structural
/// encoding requirement at once, and a lifecycle relation does the
/// same, so the boundaries are a set rather than a single value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationCoveragePlan {
    pub relation: RelationId,
    pub case: ExecutionCaseId,
    pub activity: RelationActivity,
    pub boundaries: BTreeSet<CoverageBoundary>,
    /// Canonically sorted and free of duplicates.
    pub positive: Vec<PositiveCoverageRequirement>,
    /// Canonically sorted and free of duplicates.
    pub negative: Vec<NegativeCoverageRequirement>,
    /// Present exactly for an active runtime-carried relation-case.
    pub carrier: Option<CarrierCoverageRequirement>,
    /// The accepted projection comparison of each applicable boundary.
    ///
    /// A map rather than the single value of the Guide-6 sketch: §10.3
    /// asks for a boundary-specific comparison, and a hybrid relation
    /// compares a compiler selection result and a structural fact that
    /// are not the same claim.
    pub projections: BTreeMap<CoverageBoundary, SemanticProjectionRequirement>,
    pub external_evidence: BTreeSet<ExternalEvidenceRequirement>,
}

impl RelationCoveragePlan {
    /// This plan's relation-case census key.
    #[must_use]
    pub fn key(&self) -> RelationCaseKey {
        RelationCaseKey {
            relation: self.relation.clone(),
            case: self.case.clone(),
        }
    }

    /// Every requirement identity this plan states.
    fn requirement_ids(&self) -> impl Iterator<Item = &CoverageRequirementId> {
        self.positive
            .iter()
            .map(|requirement| &requirement.id)
            .chain(self.negative.iter().map(|requirement| &requirement.id))
    }
}

/// One operation's complete coverage analysis.
///
/// Coverage is stored and validated per operation. Guide 5 proved the
/// combined placement set factorizes across operations, so an analysis
/// keyed by operation carries the same information as a cross-operation
/// product without manufacturing one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationCoverageAnalysis {
    pub operation: OperationId,
    pub cases: BTreeSet<ExecutionCaseId>,
    pub requirements: BTreeMap<RelationCaseKey, RelationCoveragePlan>,
}

/// The coverage analysis of one proof-plan candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanCoverageAnalysis {
    pub operations: BTreeMap<OperationId, OperationCoverageAnalysis>,
}

impl PlanCoverageAnalysis {
    /// Every relation-case coverage plan, across every operation.
    pub fn plans(&self) -> impl Iterator<Item = &RelationCoveragePlan> {
        self.operations
            .values()
            .flat_map(|analysis| analysis.requirements.values())
    }

    /// Every relation-case key this analysis covers.
    #[must_use]
    pub fn keys(&self) -> BTreeSet<RelationCaseKey> {
        self.plans().map(RelationCoveragePlan::key).collect()
    }

    /// The coverage plan of one relation-case, if it is covered.
    #[must_use]
    pub fn plan(&self, key: &RelationCaseKey) -> Option<&RelationCoveragePlan> {
        self.operations
            .get(&key.relation.operation())
            .and_then(|analysis| analysis.requirements.get(key))
    }
}

/// Fill the dependency collateral of every negative requirement.
///
/// The typed hook the coverage dependency graph fills: that graph is a
/// later stage, and deriving a closure here from the relation graph
/// alone would duplicate it. The supplied function receives one
/// negative requirement's own relation-case key and returns the strict
/// active-descendant closure that must become blocked.
pub fn bind_dependency_collateral(
    analysis: &mut PlanCoverageAnalysis,
    closure: &impl Fn(&RelationCaseKey) -> BTreeSet<RelationCaseKey>,
) {
    for operation in analysis.operations.values_mut() {
        for plan in operation.requirements.values_mut() {
            let key = plan.key();

            for requirement in &mut plan.negative {
                if requirement.collateral.policy
                    == CollateralPolicy::RequireIntendedAndDependencyClosure
                {
                    requirement.collateral.dependency_closure = closure(&key);
                }
            }
        }
    }
}

/// Derive one relation's coverage contract in one execution case.
///
/// # Errors
///
/// [`CompileError::SponsorValueRead`] when the relation's operand
/// census names an erased sponsor amount.
pub fn derive_relation_coverage(
    declaration: &RelationDeclaration,
    plan: &RelationCasePlan,
) -> Result<RelationCoveragePlan, CompileError> {
    let operands = relation_operands(declaration)?;
    let representation = relation_representation(&declaration.relation, &plan.case);
    let mutations = relation_mutations(&declaration.relation);

    let mut positive = Vec::new();
    let mut negative = Vec::new();

    let id = |boundary: CoverageBoundary, purpose: CoveragePurpose| CoverageRequirementId {
        relation: plan.relation.clone(),
        case: plan.case.clone(),
        boundary,
        purpose,
    };

    match plan.activity {
        // An inactive relation-case is stated explicitly and carries
        // exactly one obligation: the case is accepted and the relation
        // never activates. No mutation is required of an inactive
        // branch, because a target rejecting it would prove nothing
        // about the relation this case does not exercise.
        RelationActivity::Vacuous => {
            for boundary in &plan.boundaries {
                positive.push(PositiveCoverageRequirement {
                    id: id(*boundary, CoveragePurpose::InactiveAccept),
                    role: EvidenceRole::InactiveCaseAcceptance,
                    representation,
                    operands: Vec::new(),
                });
            }
        }

        RelationActivity::Active => {
            for boundary in &plan.boundaries {
                for (role, subject) in boundary_subjects(*boundary, plan) {
                    positive.push(PositiveCoverageRequirement {
                        id: id(*boundary, positive_purpose(*boundary, subject.as_ref())),
                        role: role.clone(),
                        representation,
                        operands: operands.clone(),
                    });

                    for mutation in boundary_mutations(&mutations, *boundary) {
                        negative.push(NegativeCoverageRequirement {
                            id: id(
                                *boundary,
                                negative_purpose(*boundary, subject.as_ref(), mutation),
                            ),
                            role: role.clone(),
                            mutation,
                            collateral: collateral(*boundary),
                        });
                    }
                }
            }
        }
    }

    positive.sort();
    negative.sort();

    Ok(RelationCoveragePlan {
        relation: plan.relation.clone(),
        case: plan.case.clone(),
        activity: plan.activity,
        boundaries: plan.boundaries.clone(),
        positive,
        negative,
        // The carrier obligation and the accepted projections are bound
        // from the plan's feasible placements, which this per-relation
        // derivation does not see.
        carrier: None,
        projections: BTreeMap::new(),
        external_evidence: plan.external_evidence.clone(),
    })
}

/// Derive and validate the coverage of one candidate's relation-case
/// plans.
///
/// # Errors
///
/// [`CompileError::CoverageCensusMismatch`] when a planned relation is
/// absent from the relation analysis; any failure of
/// [`derive_relation_coverage`] or [`validate_coverage_census`].
pub fn analyze_plan_coverage(
    relations: &CompilerRelationAnalysis,
    plans: &[RelationCasePlan],
) -> Result<PlanCoverageAnalysis, CompileError> {
    let mut operations: BTreeMap<OperationId, OperationCoverageAnalysis> = BTreeMap::new();

    for plan in plans {
        let declaration = declared_relation(relations, &plan.relation).ok_or_else(|| {
            CompileError::CoverageCensusMismatch {
                missing: vec![RelationCaseKey {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                }],
                unexpected: Vec::new(),
            }
        })?;
        let coverage = derive_relation_coverage(declaration, plan)?;
        let operation = plan.relation.operation();
        let analysis = operations
            .entry(operation)
            .or_insert_with(|| OperationCoverageAnalysis {
                operation,
                cases: BTreeSet::new(),
                requirements: BTreeMap::new(),
            });

        analysis.cases.insert(plan.case.clone());

        if analysis
            .requirements
            .insert(coverage.key(), coverage)
            .is_some()
        {
            return Err(CompileError::CoverageCensusMismatch {
                missing: Vec::new(),
                unexpected: vec![RelationCaseKey {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                }],
            });
        }
    }

    let analysis = PlanCoverageAnalysis { operations };

    validate_coverage_census(plans, &analysis)?;
    Ok(analysis)
}

/// Validate the exact relation-case coverage census.
///
/// Equality with the Guide-5 relation-case plan scope, checked against
/// the plans rather than against the derivation that produced the
/// coverage: a derivation that silently dropped a relation is caught
/// here instead of agreeing with itself.
///
/// # Errors
///
/// [`CompileError::DuplicateCoverageRequirement`] when one requirement
/// identity occurs twice; [`CompileError::CoverageCensusMismatch`] when
/// the covered relation-cases differ from the planned ones;
/// [`CompileError::MissingPositiveCoverage`] or
/// [`CompileError::MissingNegativeCoverage`] when an active
/// relation-case leaves one of its boundaries uncovered;
/// [`CompileError::MissingInactiveCoverage`] when an inactive
/// relation-case states no inactive-valid requirement;
/// [`CompileError::UnexpectedCoverageRejection`] when an inactive
/// relation-case demands a target rejection.
pub fn validate_coverage_census(
    plans: &[RelationCasePlan],
    analysis: &PlanCoverageAnalysis,
) -> Result<(), CompileError> {
    let mut seen = BTreeSet::new();

    for plan in analysis.plans() {
        for id in plan.requirement_ids() {
            if !seen.insert(id.clone()) {
                return Err(CompileError::DuplicateCoverageRequirement {
                    requirement: id.clone(),
                });
            }
        }
    }

    let covered = analysis.keys();
    let planned = plans
        .iter()
        .map(|plan| RelationCaseKey {
            relation: plan.relation.clone(),
            case: plan.case.clone(),
        })
        .collect::<BTreeSet<_>>();

    if covered != planned {
        return Err(CompileError::CoverageCensusMismatch {
            missing: planned.difference(&covered).cloned().collect(),
            unexpected: covered.difference(&planned).cloned().collect(),
        });
    }

    for plan in analysis.plans() {
        validate_relation_coverage(plan)?;
    }

    Ok(())
}

/// Validate one relation-case coverage plan against §11.4.
///
/// # Errors
///
/// Any of the per-relation failures documented on
/// [`validate_coverage_census`].
pub fn validate_relation_coverage(plan: &RelationCoveragePlan) -> Result<(), CompileError> {
    let relation = plan.relation.clone();
    let case = plan.case.clone();

    match plan.activity {
        RelationActivity::Vacuous => {
            let inactive = plan
                .positive
                .iter()
                .any(|requirement| requirement.id.purpose == CoveragePurpose::InactiveAccept);

            if !inactive {
                return Err(CompileError::MissingInactiveCoverage { relation, case });
            }

            if !plan.negative.is_empty() {
                return Err(CompileError::UnexpectedCoverageRejection { relation, case });
            }
        }

        RelationActivity::Active => {
            for boundary in &plan.boundaries {
                let at = |requirement: &CoverageRequirementId| requirement.boundary == *boundary;

                if !plan.positive.iter().any(|entry| at(&entry.id)) {
                    return Err(CompileError::MissingPositiveCoverage {
                        relation,
                        case,
                        boundary: *boundary,
                    });
                }

                if !plan.negative.iter().any(|entry| at(&entry.id)) {
                    return Err(CompileError::MissingNegativeCoverage {
                        relation,
                        case,
                        boundary: *boundary,
                    });
                }
            }
        }
    }

    Ok(())
}

/// The negative classes one relation requires, by boundary.
///
/// The Guide-6 §7.2 matrix, split into the runtime half and the
/// away-from-runtime half. Both halves match exhaustively with no
/// wildcard arm, so a new realization relation fails to compile in both
/// until its required negatives are stated rather than being absorbed
/// into a default that silently requires nothing.
pub fn relation_mutations(relation: &Relation) -> Vec<(CoverageBoundary, RelationMutation)> {
    let mut mutations = runtime_mutations(relation)
        .into_iter()
        .map(|mutation| (CoverageBoundary::RuntimeCarrier, mutation))
        .collect::<Vec<_>>();

    mutations.extend(static_mutations(relation));
    mutations
}

/// The runtime-target negatives of one relation, if it has any.
///
/// Cardinality is the one variant whose classes depend on the declared
/// values: a minimum of zero has no below-minimum mutation, because
/// "fewer than zero members" is not a world any evidence package can
/// build. Every finite maximum has an above-maximum mutation, and an
/// architecture-owned maximum is cited by its bound rather than copied.
fn runtime_mutations(relation: &Relation) -> Vec<RelationMutation> {
    use RelationMutation as Mutation;

    match relation {
        Relation::Cardinality {
            minimum, maximum, ..
        } => {
            let mut mutations = Vec::new();

            if minimum.get() > 0 {
                mutations.push(Mutation::CardinalityBelowMinimum);
            }

            mutations.push(Mutation::CardinalityAboveMaximum {
                ceiling: match maximum {
                    CardinalityMaximum::Exact(count) => CardinalityCeiling::Declared(*count),
                    CardinalityMaximum::Bound(bound) => CardinalityCeiling::Bound(*bound),
                },
            });

            mutations
        }

        Relation::AllowedObjectFamilies { .. } => vec![Mutation::UndeclaredObjectFamily],

        Relation::Recognition { .. } => vec![
            Mutation::WrongRecognizedAsset,
            Mutation::WrongRecognizedObject,
        ],

        Relation::AmountConservation { .. } => vec![Mutation::AmountMismatch],

        Relation::OwnerAuthorization { .. } => vec![Mutation::MissingRequiredOwner],

        Relation::SponsorIsolation => vec![
            Mutation::SponsorProtocolOverlap,
            Mutation::MissingSponsorAuthorization,
        ],

        Relation::SponsorEnvelopeMultiplicity { .. } => {
            vec![Mutation::SponsorEnvelopeMultiplicityExceeded]
        }

        Relation::RootPolicy { .. } => vec![Mutation::WrongRootEffect],

        Relation::ProjectionPolicy { .. } => vec![
            Mutation::MissingRequiredProjection,
            Mutation::ForbiddenProjectionPresent,
        ],

        Relation::CanonicalDeltaPolicy { .. } => vec![
            Mutation::MissingCanonicalDeltaFamily,
            Mutation::UnexpectedCanonicalDeltaFamily,
            Mutation::DuplicateCanonicalSourceOrDestination,
        ],

        Relation::OpenFlowPolicy { .. } => vec![Mutation::UndeclaredOpenFlow],

        Relation::ExpressionPredicate { .. } => vec![Mutation::ExpressionPredicateFalse],

        Relation::PermissionlessAuthorization
        | Relation::Constructibility { .. }
        | Relation::Representation { .. }
        | Relation::LifecycleExit { .. }
        | Relation::SubstrateConservation { .. } => Vec::new(),
    }
}

/// The negatives one relation requires away from the runtime boundary.
fn static_mutations(relation: &Relation) -> Vec<(CoverageBoundary, RelationMutation)> {
    use CoverageBoundary as Boundary;
    use RelationMutation as Mutation;

    match relation {
        // A hidden owner or operator gate is a property of the emitted
        // path, not a predicate any carrier evaluates.
        Relation::PermissionlessAuthorization => vec![(
            Boundary::BackendStructural,
            Mutation::UnexpectedProtocolSecret,
        )],

        Relation::Constructibility { class } => {
            let mut mutations = vec![(
                Boundary::CompilerStatic,
                Mutation::ConstructibilityWitnessUnavailable,
            )];

            if matches!(class, ConstructibilityClass::PublicPermissionless) {
                mutations.push((
                    Boundary::CompilerStatic,
                    Mutation::PermissionlessPrivateDependency,
                ));
            }

            mutations
        }

        // The selection is validated from typed input; the encoding is
        // an obligation on the emitted bundle. Two boundaries, two
        // independent negatives.
        Relation::Representation { .. } => vec![
            (
                Boundary::CompilerStatic,
                Mutation::UnsupportedRepresentation,
            ),
            (
                Boundary::BackendStructural,
                Mutation::UnauthenticatedRepresentation,
            ),
        ],

        Relation::LifecycleExit { .. } => vec![
            (
                Boundary::CompilerStatic,
                Mutation::RequiredLifecycleExitMissing,
            ),
            (
                Boundary::BackendStructural,
                Mutation::RequiredLifecycleExitMissing,
            ),
        ],

        Relation::SubstrateConservation { .. } => vec![
            (
                Boundary::ExternalEvidence,
                Mutation::ExternalEvidenceMissing,
            ),
            (Boundary::ExternalEvidence, Mutation::ExternalEvidenceFailed),
            (
                Boundary::ExternalEvidence,
                Mutation::ExternalEvidenceIdentityMismatch,
            ),
        ],

        Relation::Cardinality { .. }
        | Relation::AllowedObjectFamilies { .. }
        | Relation::Recognition { .. }
        | Relation::AmountConservation { .. }
        | Relation::OwnerAuthorization { .. }
        | Relation::SponsorIsolation
        | Relation::SponsorEnvelopeMultiplicity { .. }
        | Relation::RootPolicy { .. }
        | Relation::ProjectionPolicy { .. }
        | Relation::CanonicalDeltaPolicy { .. }
        | Relation::OpenFlowPolicy { .. }
        | Relation::ExpressionPredicate { .. } => Vec::new(),
    }
}

/// The required mutations of one boundary, canonically ordered.
fn boundary_mutations(
    mutations: &[(CoverageBoundary, RelationMutation)],
    boundary: CoverageBoundary,
) -> Vec<RelationMutation> {
    let mut selected = mutations
        .iter()
        .filter(|(owner, _)| *owner == boundary)
        .map(|(_, mutation)| *mutation)
        .collect::<Vec<_>>();

    selected.sort();
    selected.dedup();
    selected
}

/// What one boundary answers about, and who answers it.
///
/// Every boundary but the external one answers exactly once. The
/// external boundary answers once per typed evidence requirement, so
/// two requirements on one relation state two independent obligations
/// rather than collapsing into one; a relation whose external boundary
/// names no requirement states none, and the census validator reports
/// the gap instead of the derivation inventing a subject.
fn boundary_subjects(
    boundary: CoverageBoundary,
    plan: &RelationCasePlan,
) -> Vec<(EvidenceRole, Option<ExternalEvidenceRequirement>)> {
    match boundary {
        CoverageBoundary::CompilerStatic => vec![(EvidenceRole::CompilerAnalysisResult, None)],
        CoverageBoundary::BackendStructural => vec![(EvidenceRole::EmittedStructure, None)],
        CoverageBoundary::RuntimeCarrier => vec![(EvidenceRole::TargetExecution, None)],
        CoverageBoundary::ExternalEvidence => plan
            .external_evidence
            .iter()
            .map(|requirement| {
                (
                    EvidenceRole::ExternalReport {
                        requirement: requirement.clone(),
                        capability: RequiredCapability::WholeTransactionValueConservation,
                    },
                    Some(requirement.clone()),
                )
            })
            .collect(),
    }
}

/// The positive purpose of one boundary and subject.
fn positive_purpose(
    boundary: CoverageBoundary,
    subject: Option<&ExternalEvidenceRequirement>,
) -> CoveragePurpose {
    match (boundary, subject) {
        (CoverageBoundary::ExternalEvidence, Some(requirement)) => {
            CoveragePurpose::ExternalEvidenceVerified {
                requirement: requirement.clone(),
            }
        }
        (CoverageBoundary::BackendStructural, _) => CoveragePurpose::StructuralPresence,
        _ => CoveragePurpose::ActiveAccept,
    }
}

/// The negative purpose of one boundary, subject, and mutation.
fn negative_purpose(
    boundary: CoverageBoundary,
    subject: Option<&ExternalEvidenceRequirement>,
    mutation: RelationMutation,
) -> CoveragePurpose {
    match (boundary, subject) {
        (CoverageBoundary::ExternalEvidence, Some(requirement)) => {
            CoveragePurpose::ExternalEvidenceRejected {
                requirement: requirement.clone(),
                mutation,
            }
        }
        (CoverageBoundary::BackendStructural, _) => CoveragePurpose::StructuralRejection(mutation),
        _ => CoveragePurpose::FocusedReject(mutation),
    }
}

/// The collateral policy of a negative at one boundary.
///
/// A runtime mutation really does block the active relations that
/// depend on the mutated one, so their closure is required. A
/// compiler-static, structural, or external negative makes no such
/// claim: nothing here proves it is isolated either, so additional
/// collateral is reported rather than assumed absent.
const fn collateral_policy(boundary: CoverageBoundary) -> CollateralPolicy {
    match boundary {
        CoverageBoundary::RuntimeCarrier => CollateralPolicy::RequireIntendedAndDependencyClosure,
        CoverageBoundary::CompilerStatic
        | CoverageBoundary::BackendStructural
        | CoverageBoundary::ExternalEvidence => CollateralPolicy::ReportAdditional,
    }
}

const fn collateral(boundary: CoverageBoundary) -> CollateralRequirement {
    CollateralRequirement {
        policy: collateral_policy(boundary),
        dependency_closure: BTreeSet::new(),
    }
}

/// The representation the case fixed for one relation's object.
fn relation_representation(
    relation: &Relation,
    case: &ExecutionCaseId,
) -> Option<RepresentationMode> {
    match relation {
        Relation::Representation { object, .. } | Relation::LifecycleExit { object, .. } => {
            case.representations.get(object).copied()
        }
        _ => None,
    }
}

/// The declaration behind one relation ID.
fn declared_relation<'a>(
    relations: &'a CompilerRelationAnalysis,
    relation: &RelationId,
) -> Option<&'a RelationDeclaration> {
    relations
        .graph
        .node_weights()
        .find(|node| &node.source.id == relation)
        .map(|node| &node.source)
}

// --- Tranche F: carrier coverage requirements (Guide-6 §9) ---

/// Analyze the complete coverage of one placed proof-plan candidate.
///
/// The per-relation contract, plus the carrier obligation compressed
/// out of the plan's feasible placements and the accepted projection of
/// each applicable boundary.
///
/// # Errors
///
/// Any failure of [`analyze_plan_coverage`],
/// [`crate::carrier::relation_case_eligibility`],
/// [`validate_coverage_census`], or [`validate_placement_coverage`].
pub fn analyze_placed_coverage(
    relations: &CompilerRelationAnalysis,
    placed: &PlacedProofPlanCandidate,
) -> Result<PlanCoverageAnalysis, CompileError> {
    let mut analysis = analyze_plan_coverage(relations, &placed.relation_case_plans)?;
    let eligibility = relation_case_eligibility(relations, &placed.relation_case_plans)?;

    bind_carrier_coverage(&mut analysis, &eligibility, &placed.feasible_placements);
    bind_projection_coverage(&mut analysis, &placed.relation_case_plans);

    validate_coverage_census(&placed.relation_case_plans, &analysis)?;
    validate_placement_coverage(
        &eligibility,
        &placed.feasible_placements,
        &placed.layout_requirements,
        &analysis,
    )?;

    Ok(analysis)
}

/// Bind the compressed carrier obligation of every runtime
/// relation-case.
///
/// The compression is the point: one per-relation assignment appears in
/// every feasible whole-transaction placement that chose it, and
/// coverage retains it once. The multiplicity survives compression
/// because it travels inside each placed carrier — a per-member role
/// and a complete-family proof are different alternatives, never one
/// deduplicated into the other.
pub fn bind_carrier_coverage(
    analysis: &mut PlanCoverageAnalysis,
    eligibility: &[CarrierEligibility],
    placements: &[PlacementCandidate],
) {
    let alternatives = carrier_alternatives(eligibility, placements);

    for operation in analysis.operations.values_mut() {
        for (key, plan) in &mut operation.requirements {
            let Some(assignments) = alternatives.get(key) else {
                continue;
            };

            for carrier in assignments
                .iter()
                .flat_map(|alternative| alternative.carriers.iter())
            {
                plan.positive.push(PositiveCoverageRequirement {
                    id: CoverageRequirementId {
                        relation: key.relation.clone(),
                        case: key.case.clone(),
                        boundary: CoverageBoundary::RuntimeCarrier,
                        purpose: CoveragePurpose::CarrierExecution(carrier.clone()),
                    },
                    role: EvidenceRole::TargetExecution,
                    representation: None,
                    operands: Vec::new(),
                });
            }

            plan.positive.sort();
            plan.positive.dedup();
            plan.carrier = Some(CarrierCoverageRequirement {
                relation_case: key.clone(),
                allowed_assignments: assignments.clone(),
            });
        }
    }
}

/// The eligible carriers of each relation-case, indexed by key.
///
/// The placement product is scanned once per assignment, so a linear
/// search here would be quadratic in a scope whose placements number in
/// the tens of thousands.
fn eligibility_index(
    eligibility: &[CarrierEligibility],
) -> BTreeMap<RelationCaseKey, &CarrierEligibility> {
    eligibility
        .iter()
        .map(|entry| {
            (
                RelationCaseKey {
                    relation: entry.relation.clone(),
                    case: entry.case.clone(),
                },
                entry,
            )
        })
        .collect()
}

/// One assignment as it appears in the placement product.
///
/// Borrowed rather than owned so the repeat filter allocates nothing:
/// the same per-relation assignment recurs in every whole-transaction
/// placement that chose it, which is exactly what the compression
/// exists to collapse.
type AssignmentRef<'a> = (&'a RelationId, &'a ExecutionCaseId, &'a [PlacedCarrier]);

/// The deduplicated assignment alternatives of every relation-case.
///
/// One relation-case assignment recurs once per combined placement that
/// selected it — 216 placements of one pilot plan repeat a handful of
/// distinct assignments — so each distinct assignment is expanded once
/// and every repeat is skipped before any layout dependency is derived.
fn carrier_alternatives(
    eligibility: &[CarrierEligibility],
    placements: &[PlacementCandidate],
) -> BTreeMap<RelationCaseKey, BTreeSet<CarrierAssignmentAlternative>> {
    let index = eligibility_index(eligibility);
    let mut alternatives: BTreeMap<RelationCaseKey, BTreeSet<CarrierAssignmentAlternative>> =
        BTreeMap::new();
    let mut seen: BTreeSet<AssignmentRef<'_>> = BTreeSet::new();

    for placement in placements {
        for assignment in &placement.assignments {
            if !seen.insert((
                &assignment.relation,
                &assignment.case,
                assignment.carriers.as_slice(),
            )) {
                continue;
            }

            let key = assignment.key();
            let Some(carriers) = index.get(&key) else {
                continue;
            };

            alternatives
                .entry(key)
                .or_default()
                .insert(assignment_alternative(carriers, &assignment.carriers));
        }
    }

    alternatives
}

/// One placed assignment with the layout requirements it depends on.
fn assignment_alternative(
    carriers: &CarrierEligibility,
    placed: &[PlacedCarrier],
) -> CarrierAssignmentAlternative {
    let mut layout = BTreeSet::new();

    for carrier in placed {
        let entry = carriers.eligible.iter().find(|entry| {
            entry.carrier == carrier.carrier && entry.quantification == carrier.quantification
        });

        if let Some(entry) = entry {
            layout.extend(selected_carrier_requirements(carriers, entry));
        }
    }

    CarrierAssignmentAlternative {
        carriers: placed.iter().cloned().collect(),
        layout,
    }
}

// --- Tranche G: accepted semantic projections (Guide-6 §10) ---

/// Bind the accepted projection of every active relation-case.
///
/// One projection per applicable boundary, because the comparison
/// differs by boundary: a runtime relation compares its verdict and its
/// public operands, a compiler-static one compares the compiler's own
/// selection result, a structural one compares an emitted fact, and an
/// external one compares an evidence role and subject.
pub fn bind_projection_coverage(analysis: &mut PlanCoverageAnalysis, plans: &[RelationCasePlan]) {
    let sources = plans
        .iter()
        .map(|plan| {
            (
                RelationCaseKey {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                },
                plan,
            )
        })
        .collect::<BTreeMap<_, _>>();

    for operation in analysis.operations.values_mut() {
        for (key, plan) in &mut operation.requirements {
            if plan.activity != RelationActivity::Active {
                continue;
            }

            let Some(source) = sources.get(key) else {
                continue;
            };
            let derived = plan
                .boundaries
                .iter()
                .filter_map(|boundary| {
                    projection_requirement(key, plan, source, *boundary)
                        .map(|requirement| (boundary_role(plan, *boundary), requirement))
                })
                .collect::<Vec<_>>();

            for (role, requirement) in derived {
                plan.positive.push(PositiveCoverageRequirement {
                    id: CoverageRequirementId {
                        relation: key.relation.clone(),
                        case: key.case.clone(),
                        boundary: requirement.boundary,
                        purpose: CoveragePurpose::AcceptedProjection,
                    },
                    role,
                    representation: None,
                    operands: requirement.operands.clone(),
                });
                plan.projections.insert(requirement.boundary, requirement);
            }

            plan.positive.sort();
            plan.positive.dedup();
        }
    }
}

/// One boundary's accepted projection requirement, if it has one.
fn projection_requirement(
    key: &RelationCaseKey,
    plan: &RelationCoveragePlan,
    source: &RelationCasePlan,
    boundary: CoverageBoundary,
) -> Option<SemanticProjectionRequirement> {
    let subject = projection_subject(boundary, plan)?;

    Some(SemanticProjectionRequirement {
        relation: key.relation.clone(),
        case: key.case.clone(),
        boundary,
        compare_relation_verdict: subject == ProjectionSubject::RuntimeRelationVerdict,
        subject,
        operands: boundary_operands(plan, boundary),
        sources: boundary_sources(source, boundary),
    })
}

/// What one boundary's accepted projection compares.
fn projection_subject(
    boundary: CoverageBoundary,
    plan: &RelationCoveragePlan,
) -> Option<ProjectionSubject> {
    match boundary {
        CoverageBoundary::RuntimeCarrier => Some(ProjectionSubject::RuntimeRelationVerdict),
        CoverageBoundary::CompilerStatic => Some(ProjectionSubject::CompilerSelectionResult),
        CoverageBoundary::BackendStructural => Some(ProjectionSubject::EmittedStructuralFact),
        CoverageBoundary::ExternalEvidence => {
            if plan.external_evidence.is_empty() {
                None
            } else {
                Some(ProjectionSubject::ExternalReportSubjects {
                    requirements: plan.external_evidence.clone(),
                })
            }
        }
    }
}

/// The positive requirement that answers one boundary, if it has one.
fn boundary_answer(
    plan: &RelationCoveragePlan,
    boundary: CoverageBoundary,
) -> Option<&PositiveCoverageRequirement> {
    plan.positive
        .iter()
        .find(|requirement| requirement.id.boundary == boundary)
}

/// The operand census one boundary's positive coverage already states.
fn boundary_operands(plan: &RelationCoveragePlan, boundary: CoverageBoundary) -> Vec<OperandId> {
    boundary_answer(plan, boundary)
        .map(|requirement| requirement.operands.clone())
        .unwrap_or_default()
}

/// Who answers one boundary's accepted projection.
///
/// The same evidence role that answers the boundary's acceptance, never
/// a fixed target execution: a compiler-static selection result, an
/// emitted structural fact, and an external report are not target
/// executions, and claiming one would report an obligation no target
/// evaluates as one it did. Where an external boundary names several
/// reports the comparison covers all of them and the role names the
/// first; the role of a boundary with no positive requirement at all is
/// unreachable, because such a boundary states no projection either.
fn boundary_role(plan: &RelationCoveragePlan, boundary: CoverageBoundary) -> EvidenceRole {
    boundary_answer(plan, boundary).map_or(EvidenceRole::TargetExecution, |requirement| {
        requirement.role.clone()
    })
}

/// The active source rows one boundary's projection covers.
///
/// Only the runtime boundary has carrier-routed sources: a
/// compiler-static, structural, or external obligation is not answered
/// by a fact routed to a carrier, and listing rows there would imply a
/// routing nothing performs.
fn boundary_sources(plan: &RelationCasePlan, boundary: CoverageBoundary) -> Vec<SourceRequirement> {
    if boundary != CoverageBoundary::RuntimeCarrier {
        return Vec::new();
    }

    let mut sources = plan
        .runtime_requirements
        .iter()
        .flat_map(|requirement| requirement.sources.iter().cloned())
        .collect::<Vec<_>>();

    sources.sort();
    sources.dedup();
    sources
}

/// True for a coverage analysis that names an erased sponsor value.
///
/// The coverage-specific traversal of §10.2: sponsor family membership,
/// owner authorization, disjointness, and envelope multiplicity all
/// remain expressible, and the individual amount does not — not as an
/// operand, not as a projected source row, and not through a layout
/// requirement a carrier alternative depends on.
#[must_use]
pub fn coverage_names_sponsor_value(analysis: &PlanCoverageAnalysis) -> bool {
    analysis.plans().any(|plan| {
        let stated = plan
            .positive
            .iter()
            .flat_map(|requirement| requirement.operands.iter())
            .chain(
                plan.projections
                    .values()
                    .flat_map(|requirement| requirement.operands.iter()),
            )
            .any(|operand| is_sponsor_amount_operand(operand.role()));
        let projected = plan
            .projections
            .values()
            .flat_map(|requirement| requirement.sources.iter())
            .any(|source| is_sponsor_amount_operand(source.operand.role()));
        let routed = plan
            .carrier
            .iter()
            .flat_map(|carrier| carrier.allowed_assignments.iter())
            .flat_map(|alternative| alternative.layout.iter())
            .any(names_sponsor_amount);

        stated || projected || routed
    })
}

/// Reject a coverage analysis that names an erased sponsor value.
///
/// # Errors
///
/// [`CompileError::SponsorValueRead`] on any occurrence.
pub fn validate_sponsor_erasure(analysis: &PlanCoverageAnalysis) -> Result<(), CompileError> {
    if coverage_names_sponsor_value(analysis) {
        return Err(CompileError::SponsorValueRead);
    }

    Ok(())
}

// --- Tranche H: placement, layout, and plan-set censuses ---

/// Validate the placement, layout, and projection coverage of one plan.
///
/// # Errors
///
/// [`CompileError::MissingCarrierCoverage`] when an active runtime
/// relation-case has no carrier obligation;
/// [`CompileError::UnexpectedRuntimeCarrierCoverage`] when a
/// compiler-static, backend-structural, external, or inactive
/// relation-case has one; [`CompileError::MissingProjectionCoverage`]
/// when runtime-target acceptance is claimed with no accepted
/// projection; [`CompileError::MissingCoverageLayoutRequirement`] when
/// a selectable assignment or its layout dependencies are unreferenced;
/// [`CompileError::UnexpectedCoverageLayoutRequirement`] when coverage
/// references a layout requirement the plan's census does not state;
/// [`CompileError::SponsorValueRead`] on any erased sponsor value.
pub fn validate_placement_coverage(
    eligibility: &[CarrierEligibility],
    placements: &[PlacementCandidate],
    layout: &[LayoutRequirement],
    analysis: &PlanCoverageAnalysis,
) -> Result<(), CompileError> {
    let carried = eligibility
        .iter()
        .map(|entry| RelationCaseKey {
            relation: entry.relation.clone(),
            case: entry.case.clone(),
        })
        .collect::<BTreeSet<_>>();
    let stated = layout.iter().collect::<BTreeSet<_>>();

    for plan in analysis.plans() {
        let key = plan.key();
        let runtime = plan.activity == RelationActivity::Active
            && plan.boundaries.contains(&CoverageBoundary::RuntimeCarrier)
            && carried.contains(&key);

        match (&plan.carrier, runtime) {
            (Some(_), false) => {
                return Err(CompileError::UnexpectedRuntimeCarrierCoverage {
                    relation: key.relation,
                    case: key.case,
                });
            }
            (None, true) => {
                return Err(CompileError::MissingCarrierCoverage {
                    relation: key.relation,
                    case: key.case,
                });
            }
            _ => {}
        }

        if runtime
            && !plan
                .projections
                .contains_key(&CoverageBoundary::RuntimeCarrier)
        {
            return Err(CompileError::MissingProjectionCoverage {
                relation: key.relation,
                case: key.case,
            });
        }

        for alternative in plan
            .carrier
            .iter()
            .flat_map(|carrier| carrier.allowed_assignments.iter())
        {
            for requirement in &alternative.layout {
                if !stated.contains(requirement) {
                    return Err(CompileError::UnexpectedCoverageLayoutRequirement {
                        relation: key.relation.clone(),
                        case: key.case,
                    });
                }
            }
        }
    }

    // Every selectable assignment is covered, with exactly the layout
    // requirements that assignment depends on: recomputed from the
    // placement rather than trusted because the compression produced
    // it. A repeat of an assignment already checked is skipped rather
    // than recomputed — the same inputs cannot reach a different
    // verdict, and the product repeats each distinct assignment once
    // per placement that chose it.
    let index = eligibility_index(eligibility);
    let mut checked: BTreeSet<AssignmentRef<'_>> = BTreeSet::new();

    for placement in placements {
        for assignment in &placement.assignments {
            if !checked.insert((
                &assignment.relation,
                &assignment.case,
                assignment.carriers.as_slice(),
            )) {
                continue;
            }

            let key = assignment.key();
            let Some(carriers) = index.get(&key) else {
                continue;
            };
            let required = assignment_alternative(carriers, &assignment.carriers);
            let covered = analysis
                .plan(&key)
                .and_then(|plan| plan.carrier.as_ref())
                .ok_or_else(|| CompileError::MissingCarrierCoverage {
                    relation: key.relation.clone(),
                    case: key.case.clone(),
                })?;

            if !covered.allowed_assignments.contains(&required) {
                return Err(CompileError::MissingCoverageLayoutRequirement {
                    relation: key.relation,
                    case: key.case,
                });
            }
        }
    }

    validate_sponsor_erasure(analysis)
}

/// Validate the conditional coverage triplet across a plan set.
///
/// A relation inactive in one case and active in another must carry all
/// three rows somewhere in the complete case set: inactive valid,
/// active valid, and active invalid. Two of the three would leave a
/// conditional relation provable only in whichever branch happens to be
/// convenient.
///
/// # Errors
///
/// [`CompileError::MissingPositiveCoverage`] or
/// [`CompileError::MissingNegativeCoverage`] when a conditional
/// relation lacks its active rows.
#[allow(dead_code)]
pub fn validate_conditional_coverage(
    analyses: &[PlanCoverageAnalysis],
) -> Result<(), CompileError> {
    let mut inactive: BTreeMap<RelationId, ExecutionCaseId> = BTreeMap::new();
    let mut accepted = BTreeSet::new();
    let mut rejected = BTreeSet::new();

    for analysis in analyses {
        for plan in analysis.plans() {
            match plan.activity {
                RelationActivity::Vacuous => {
                    inactive.insert(plan.relation.clone(), plan.case.clone());
                }
                RelationActivity::Active => {
                    if plan
                        .positive
                        .iter()
                        .any(|requirement| requirement.id.purpose == CoveragePurpose::ActiveAccept)
                    {
                        accepted.insert(plan.relation.clone());
                    }

                    if !plan.negative.is_empty() {
                        rejected.insert(plan.relation.clone());
                    }
                }
            }
        }
    }

    for (relation, case) in inactive {
        if !accepted.contains(&relation) {
            return Err(CompileError::MissingPositiveCoverage {
                relation,
                case,
                boundary: CoverageBoundary::RuntimeCarrier,
            });
        }

        if !rejected.contains(&relation) {
            return Err(CompileError::MissingNegativeCoverage {
                relation,
                case,
                boundary: CoverageBoundary::RuntimeCarrier,
            });
        }
    }

    Ok(())
}

/// Validate the representation census across a complete plan set.
///
/// Each proof-plan candidate fixes one mode, so representation coverage
/// is distributed across the plan set rather than crossed inside one
/// plan: a case fixing two modes at once would be a world no plan
/// admits.
///
/// # Errors
///
/// [`CompileError::MissingRepresentationCoverage`] when an allowed mode
/// of a representation relation is covered by no plan in the set.
#[allow(dead_code)]
pub fn validate_representation_coverage(
    relations: &CompilerRelationAnalysis,
    analyses: &[PlanCoverageAnalysis],
) -> Result<(), CompileError> {
    let mut covered: BTreeMap<RelationId, BTreeSet<RepresentationMode>> = BTreeMap::new();

    for analysis in analyses {
        for plan in analysis.plans() {
            for requirement in &plan.positive {
                if let Some(representation) = requirement.representation {
                    covered
                        .entry(plan.relation.clone())
                        .or_default()
                        .insert(representation);
                }
            }
        }
    }

    for node in relations.graph.node_weights() {
        let Relation::Representation { allowed, .. } = &node.source.relation else {
            continue;
        };
        let seen = covered.get(&node.source.id).cloned().unwrap_or_default();

        for representation in allowed {
            if !seen.contains(representation) {
                return Err(CompileError::MissingRepresentationCoverage {
                    relation: node.source.id.clone(),
                    representation: *representation,
                });
            }
        }
    }

    Ok(())
}

// --- §16 stable projection ---

/// The stable projection of one relation-case coverage plan.
///
/// Set- and map-shaped by construction, so no vector position or
/// derivation order can reach a comparison.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelationCoverageProjection {
    pub activity: RelationActivity,
    pub boundaries: BTreeSet<CoverageBoundary>,
    pub positive: BTreeSet<PositiveCoverageRequirement>,
    pub negative: BTreeSet<NegativeCoverageRequirement>,
    pub carrier: BTreeSet<CarrierAssignmentAlternative>,
    pub projections: BTreeMap<CoverageBoundary, SemanticProjectionRequirement>,
    pub external_evidence: BTreeSet<ExternalEvidenceRequirement>,
}

/// The stable projection of one plan's coverage analysis.
///
/// Typed semantic values only. Petgraph indices, search-state counts,
/// the combined placement product order, target program identities,
/// target positions, timestamps, paths, and digests are all absent —
/// the last of these because the crate mints none.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct CoverageProjection {
    pub requirements: BTreeMap<RelationCaseKey, RelationCoverageProjection>,
}

impl RelationCoveragePlan {
    /// This plan's stable projection.
    #[must_use]
    pub fn project(&self) -> RelationCoverageProjection {
        RelationCoverageProjection {
            activity: self.activity,
            boundaries: self.boundaries.clone(),
            positive: self.positive.iter().cloned().collect(),
            negative: self.negative.iter().cloned().collect(),
            carrier: self
                .carrier
                .iter()
                .flat_map(|carrier| carrier.allowed_assignments.iter().cloned())
                .collect(),
            projections: self.projections.clone(),
            external_evidence: self.external_evidence.clone(),
        }
    }
}

impl PlanCoverageAnalysis {
    /// This analysis's stable projection.
    #[must_use]
    #[allow(dead_code)]
    pub fn project(&self) -> CoverageProjection {
        CoverageProjection {
            requirements: self
                .plans()
                .map(|plan| (plan.key(), plan.project()))
                .collect(),
        }
    }
}

/// The stable projection of one operation's coverage.
///
/// The unit the factorization argument is made in: an operation's
/// coverage is a complete value on its own, so two scopes can be
/// compared operation by operation without either side ever building a
/// cross-operation product to compare.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperationCoverageProjection {
    pub operation: OperationId,
    pub cases: BTreeSet<ExecutionCaseId>,
    pub requirements: BTreeMap<RelationCaseKey, RelationCoverageProjection>,
}

impl OperationCoverageAnalysis {
    /// This operation's stable projection.
    #[must_use]
    pub fn project(&self) -> OperationCoverageProjection {
        OperationCoverageProjection {
            operation: self.operation,
            cases: self.cases.clone(),
            requirements: self
                .requirements
                .iter()
                .map(|(key, plan)| (key.clone(), plan.project()))
                .collect(),
        }
    }
}

// --- Guide-6 Wave 5: scope-level coverage (§11, §15, §16) ---

/// One proof plan's coverage together with its resolved dependencies.
///
/// The graph travels as its stable projection rather than as the graph
/// itself: the Petgraph handles are a local artifact of how the graph
/// was built, and an analysis that carried them could compare equal to
/// itself and unequal to an identical analysis built in another order.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct PlanCoverage {
    pub coverage: PlanCoverageAnalysis,
    pub dependencies: CoverageGraphProjection,
}

/// The coverage of a complete feasible proof-plan set.
///
/// Keyed by the complete typed proof plan, matching the placement
/// analysis it consumes: until an admitted plan identity exists, the
/// typed value is the only honest outer key, and a vector position or
/// candidate number would make two equal analyses compare unequal.
///
/// Nothing here is a cross-operation product. Each plan's coverage stays
/// stored per operation, so a two-operation scope carries two operation
/// analyses rather than one analysis over paired cases.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ScopeCoverageAnalysis {
    pub plans: BTreeMap<ProofPlanCandidate, PlanCoverage>,
}

/// The stable projection of one scope's coverage.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct PlanCoverageProjection {
    pub coverage: CoverageProjection,
    pub dependencies: CoverageGraphProjection,
}

/// The stable projection of a complete scope coverage analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ScopeCoverageProjection {
    pub plans: BTreeMap<ProofPlanCandidate, PlanCoverageProjection>,
}

impl PlanCoverage {
    /// This plan's stable projection.
    #[must_use]
    #[allow(dead_code)]
    pub fn project(&self) -> PlanCoverageProjection {
        PlanCoverageProjection {
            coverage: self.coverage.project(),
            dependencies: self.dependencies.clone(),
        }
    }
}

impl ScopeCoverageAnalysis {
    /// This scope's stable projection.
    #[must_use]
    #[allow(dead_code)]
    pub fn project(&self) -> ScopeCoverageProjection {
        ScopeCoverageProjection {
            plans: self
                .plans
                .iter()
                .map(|(plan, coverage)| (plan.clone(), coverage.project()))
                .collect(),
        }
    }

    /// Every plan's coverage analysis, in stable plan order.
    #[allow(dead_code)]
    pub fn analyses(&self) -> impl Iterator<Item = &PlanCoverageAnalysis> {
        self.plans.values().map(|entry| &entry.coverage)
    }

    /// The distinct per-operation coverage projections of one operation.
    ///
    /// A set of factors, never a product: two scopes agree on an
    /// operation when they state the same factors for it, and the
    /// comparison neither pairs plans across scopes nor multiplies one
    /// operation's coverage by another's.
    #[must_use]
    #[allow(dead_code)]
    pub fn operation_projections(
        &self,
        operation: OperationId,
    ) -> BTreeSet<OperationCoverageProjection> {
        self.analyses()
            .filter_map(|analysis| analysis.operations.get(&operation))
            .map(OperationCoverageAnalysis::project)
            .collect()
    }

    /// Every execution case this scope covers.
    #[must_use]
    #[allow(dead_code)]
    pub fn cases(&self) -> BTreeSet<ExecutionCaseId> {
        self.analyses()
            .flat_map(|analysis| analysis.operations.values())
            .flat_map(|operation| operation.cases.iter().cloned())
            .collect()
    }
}

/// Analyze the coverage of a complete placed proof-plan set.
///
/// Every feasible plan runs the whole way through: relation-case
/// coverage, the carrier obligations compressed out of its feasible
/// placements, the accepted projections of each boundary, and the typed
/// dependency graph whose descendant closures bind the collateral of
/// every runtime negative. The plan-set censuses — the conditional
/// triplet and the representation census — are then validated over the
/// assembled collection, because neither is a property any single plan
/// can satisfy alone.
///
/// # Errors
///
/// [`CompileError::DuplicatePlacedProofPlan`] when one proof plan is
/// placed twice; [`CompileError::CoverageCensusMismatch`] when a plan's
/// coverage scope differs from its relations crossed with its cases;
/// [`CompileError::ExecutionCaseCensusMismatch`] when the covered cases
/// differ from the placed case census; any failure of
/// [`analyze_placed_coverage`],
/// [`crate::coverage_graph::resolve_coverage_dependencies`],
/// [`validate_conditional_coverage`], or
/// [`validate_representation_coverage`].
#[allow(dead_code)]
pub fn analyze_scope_coverage(
    relations: &CompilerRelationAnalysis,
    placed: &PlacedProofPlans,
) -> Result<ScopeCoverageAnalysis, CompileError> {
    let mut plans: BTreeMap<ProofPlanCandidate, PlanCoverage> = BTreeMap::new();

    for entry in &placed.placed {
        let mut coverage = analyze_placed_coverage(relations, entry)?;
        let graph = resolve_coverage_dependencies(&mut coverage, relations)?;

        validate_plan_coverage_scope(relations, entry, &coverage)?;

        let plan = PlanCoverage {
            coverage,
            dependencies: graph.project(),
        };

        if plans.insert(entry.proof_plan.clone(), plan).is_some() {
            return Err(CompileError::DuplicatePlacedProofPlan);
        }
    }

    let analysis = ScopeCoverageAnalysis { plans };

    validate_scope_coverage(relations, placed, &analysis)?;
    Ok(analysis)
}

/// Validate one plan's coverage scope against its relations and cases
/// (§11.1).
///
/// The expected scope is derived from the relation analysis and the
/// plan's own execution cases rather than from the relation-case plans
/// the coverage was built from: a stage that dropped a relation before
/// coverage ever saw it would otherwise agree with the coverage that
/// inherited the gap.
#[allow(dead_code)]
fn validate_plan_coverage_scope(
    relations: &CompilerRelationAnalysis,
    placed: &PlacedProofPlanCandidate,
    coverage: &PlanCoverageAnalysis,
) -> Result<(), CompileError> {
    let expected = relations
        .graph
        .node_weights()
        .flat_map(|node| {
            placed
                .execution_cases
                .iter()
                .filter(move |case| case.id.operation == node.source.id.operation())
                .map(move |case| RelationCaseKey {
                    relation: node.source.id.clone(),
                    case: case.id.clone(),
                })
        })
        .collect::<BTreeSet<_>>();
    let covered = coverage.keys();

    if covered == expected {
        return Ok(());
    }

    Err(CompileError::CoverageCensusMismatch {
        missing: expected.difference(&covered).cloned().collect(),
        unexpected: covered.difference(&expected).cloned().collect(),
    })
}

/// Validate the censuses only the complete plan set can carry.
///
/// # Errors
///
/// [`CompileError::ExecutionCaseCensusMismatch`] when the covered cases
/// differ from the placed case census; any failure of
/// [`validate_conditional_coverage`] or
/// [`validate_representation_coverage`].
#[allow(dead_code)]
pub fn validate_scope_coverage(
    relations: &CompilerRelationAnalysis,
    placed: &PlacedProofPlans,
    analysis: &ScopeCoverageAnalysis,
) -> Result<(), CompileError> {
    let expected = placed.execution_case_census();
    let covered = analysis.cases();

    if covered != expected {
        return Err(CompileError::ExecutionCaseCensusMismatch {
            missing: expected.difference(&covered).cloned().collect(),
            unexpected: covered.difference(&expected).cloned().collect(),
        });
    }

    let per_plan = analysis.analyses().cloned().collect::<Vec<_>>();

    validate_conditional_coverage(&per_plan)?;
    validate_representation_coverage(relations, &per_plan)
}
