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

// The analysis stages have no non-test consumer until the P2-012
// analyzed program; unit tests exercise them until then. Remove with
// the first real consumer.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use architecture::{BoundId, OperationId};
use realization::{
    CardinalityMaximum, ConstructibilityClass, Count, ExternalEvidenceRequirement, Relation,
    RelationDeclaration, RelationId, RepresentationMode,
};

use crate::{
    CompileError,
    capability::RequiredCapability,
    case::ExecutionCaseId,
    placement::{RelationActivity, RelationCaseKey, RelationCasePlan},
    relation::CompilerRelationAnalysis,
    source::{OperandId, relation_operands},
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
    CarrierExecution(crate::placement::PlacedCarrier),
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
