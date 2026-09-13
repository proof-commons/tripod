//! Independent coverage oracle (Guide-6 Tranche I).
//!
//! Coverage derivation is an exact census over a finite product, so a
//! test that agrees with its own output proves nothing. This oracle
//! restates the Guide-6 §15 acceptance matrices, the §7.2 mutation
//! catalogue, the §6 boundary rules, the §9 carrier compression, and the
//! §10 accepted projections from scratch, builds the complete expected
//! [`CoverageProjection`], and compares the whole value with production.
//!
//! Independence is the point of the file. Nothing here calls
//! `derive_relation_coverage`, `analyze_plan_coverage` (except as the
//! subject under comparison), `relation_mutations`, the carrier or
//! projection binders, the census validators, the coverage definition
//! census, the dependency derivation, or the Petgraph closure and SCC
//! analysis to *form an expectation*. Every rule those stages apply is
//! written out again here. What the oracle does share is the upstream
//! Guide-5 input the coverage stage itself consumes — the relation
//! graph, the proof-plan candidate, the execution cases, the
//! relation-case plans, carrier eligibility, and the feasible placements
//! — because coverage is a function of those, and re-deriving them would
//! be testing a different stage.
//!
//! The defect cases corrupt a derived analysis and assert twice: the
//! production validator rejects it, and the oracle's independent
//! expectation disagrees with it. A corruption only production notices
//! would mean the oracle is blind; one only the oracle notices would
//! mean the validator is.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OpenFlowKind, OperationId};
use proptest::prelude::*;
use realization::{
    CardinalityMaximum, ConstructibilityClass, ExternalEvidenceRequirement, Relation,
    RelationDeclaration, RelationId, RelationKind, RelationSubject, RepresentationMode,
    TransactionSide,
};

use super::bound_input;
use crate::{
    CompileError,
    capability::{CapabilityView, RequiredCapability},
    carrier::{CarrierEligibility, CarrierQuantification, CarrierRole, EligibleCarrier},
    case::{ExecutionCaseId, SponsorCase},
    coverage::{
        CardinalityCeiling, CarrierAssignmentAlternative, CollateralPolicy, CollateralRequirement,
        CoverageBoundary, CoverageProjection, CoveragePurpose, CoverageRequirementId, EvidenceRole,
        NegativeCoverageRequirement, PlanCoverageAnalysis, PositiveCoverageRequirement,
        ProjectionSubject, RelationCoveragePlan, RelationCoverageProjection, RelationMutation,
        SemanticProjectionRequirement, analyze_placed_coverage, analyze_plan_coverage,
        validate_conditional_coverage, validate_coverage_census, validate_placement_coverage,
        validate_representation_coverage,
    },
    coverage_graph::{
        CoverageCycleComponent, CoverageDefinitionCensus, CoverageDependency,
        CoverageDependencyGraph, CoverageEdge, CoverageNode, CoverageNodeId,
        analyze_coverage_dependencies, build_coverage_graph, resolve_coverage_dependencies,
    },
    layout::LayoutRequirement,
    placement::{
        PlacedCarrier, PlacedProofPlanCandidate, PlacementCandidate, PlacementSearchLimits,
        RelationActivity, RelationCaseKey, RelationCasePlan, classify_relation_cases,
        place_feasible_proof_plans,
    },
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::{AnalysisNodeId, CompilerRelationAnalysis},
    source::{OperandId, OperandRole, SourceRequirement, relation_operands},
};

// --- §12.2 the direct relation × case oracle ---

/// The §15 acceptance-matrix boundaries of one relation, restated.
///
/// Read off the two pilot tables rather than derived: a compiler-static
/// obligation is answered by the compiler's own analysis, a structural
/// one by the emitted bundle, an external one by a typed report, and
/// everything a target actually evaluates is runtime. A representation
/// or lifecycle relation is genuinely hybrid and states two.
fn oracle_boundaries(relation: &Relation, case: &ExecutionCaseId) -> BTreeSet<CoverageBoundary> {
    use CoverageBoundary as Boundary;

    if oracle_committed_amounts(relation, case) {
        return BTreeSet::from([Boundary::ExternalEvidence]);
    }

    match relation {
        Relation::SubstrateConservation { .. }
        | Relation::OperatorAuthorization
        | Relation::Constructibility {
            class: realization::ConstructibilityClass::Operator,
        } => BTreeSet::from([Boundary::ExternalEvidence]),
        Relation::Constructibility { .. } => BTreeSet::from([Boundary::CompilerStatic]),
        Relation::PermissionlessAuthorization => BTreeSet::from([Boundary::BackendStructural]),
        Relation::Representation { .. } | Relation::LifecycleExit { .. } => {
            BTreeSet::from([Boundary::CompilerStatic, Boundary::BackendStructural])
        }
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
        | Relation::ExpressionPredicate { .. } => BTreeSet::from([Boundary::RuntimeCarrier]),
    }
}

/// Whether one relation activates in one execution case, restated.
///
/// The §15 conditional rows: a relation over the optional sponsor family
/// exists only where that family does. Sponsor isolation and sponsor
/// envelope multiplicity are unconditional — with no sponsor region they
/// establish that none exists, which is a claim about every case.
fn oracle_activity(relation: &Relation, case: &ExecutionCaseId) -> RelationActivity {
    let conditional = match relation {
        Relation::Cardinality { object, .. } | Relation::Recognition { object, .. } => {
            *object == ObjectId::PlainLbtc
        }
        _ => false,
    };

    if conditional && case.sponsor != SponsorCase::Present {
        RelationActivity::Vacuous
    } else {
        RelationActivity::Active
    }
}

/// The negative classes one relation requires, by boundary, restated.
///
/// The §7.2 matrix and the §7.3 cardinality edges written out again: a
/// minimum of zero has no below-minimum class, every finite maximum has
/// an above-maximum class, and an architecture-owned ceiling is cited by
/// its bound rather than copied as a value.
#[allow(clippy::too_many_lines)]
fn oracle_mutations(
    relation: &Relation,
    case: &ExecutionCaseId,
) -> BTreeSet<(CoverageBoundary, RelationMutation)> {
    use CoverageBoundary as Boundary;
    use RelationMutation as Mutation;

    if oracle_committed_amounts(relation, case) {
        return BTreeSet::from([
            (
                Boundary::ExternalEvidence,
                Mutation::ExternalEvidenceMissing,
            ),
            (Boundary::ExternalEvidence, Mutation::ExternalEvidenceFailed),
            (
                Boundary::ExternalEvidence,
                Mutation::ExternalEvidenceIdentityMismatch,
            ),
        ]);
    }

    let runtime = |mutations: Vec<Mutation>| {
        mutations
            .into_iter()
            .map(|mutation| (Boundary::RuntimeCarrier, mutation))
            .collect::<BTreeSet<_>>()
    };

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

            runtime(mutations)
        }

        Relation::AllowedObjectFamilies { .. } => runtime(vec![Mutation::UndeclaredObjectFamily]),

        Relation::Recognition { .. } => runtime(vec![
            Mutation::WrongRecognizedAsset,
            Mutation::WrongRecognizedObject,
        ]),

        Relation::AmountConservation { .. } => runtime(vec![Mutation::AmountMismatch]),

        Relation::OwnerAuthorization { .. } => runtime(vec![Mutation::MissingRequiredOwner]),

        Relation::SponsorIsolation => runtime(vec![
            Mutation::SponsorProtocolOverlap,
            Mutation::MissingSponsorAuthorization,
        ]),

        Relation::SponsorEnvelopeMultiplicity { .. } => {
            runtime(vec![Mutation::SponsorEnvelopeMultiplicityExceeded])
        }

        Relation::RootPolicy { .. } => runtime(vec![Mutation::WrongRootEffect]),

        Relation::ProjectionPolicy { .. } => runtime(vec![
            Mutation::MissingRequiredProjection,
            Mutation::ForbiddenProjectionPresent,
        ]),

        Relation::CanonicalDeltaPolicy { .. } => runtime(vec![
            Mutation::MissingCanonicalDeltaFamily,
            Mutation::UnexpectedCanonicalDeltaFamily,
            Mutation::DuplicateCanonicalSourceOrDestination,
        ]),

        Relation::OpenFlowPolicy { .. } => runtime(vec![Mutation::UndeclaredOpenFlow]),

        Relation::ExpressionPredicate { .. } => runtime(vec![Mutation::ExpressionPredicateFalse]),

        Relation::PermissionlessAuthorization => BTreeSet::from([(
            Boundary::BackendStructural,
            Mutation::UnexpectedProtocolSecret,
        )]),

        Relation::SubstrateConservation { .. }
        | Relation::OperatorAuthorization
        | Relation::Constructibility {
            class: ConstructibilityClass::Operator,
        } => BTreeSet::from([
            (
                Boundary::ExternalEvidence,
                Mutation::ExternalEvidenceMissing,
            ),
            (Boundary::ExternalEvidence, Mutation::ExternalEvidenceFailed),
            (
                Boundary::ExternalEvidence,
                Mutation::ExternalEvidenceIdentityMismatch,
            ),
        ]),

        Relation::Constructibility { class } => {
            let mut mutations = BTreeSet::from([(
                Boundary::CompilerStatic,
                Mutation::ConstructibilityWitnessUnavailable,
            )]);

            if matches!(class, ConstructibilityClass::PublicPermissionless) {
                mutations.insert((
                    Boundary::CompilerStatic,
                    Mutation::PermissionlessPrivateDependency,
                ));
            }

            mutations
        }

        Relation::Representation { .. } => BTreeSet::from([
            (
                Boundary::CompilerStatic,
                Mutation::UnsupportedRepresentation,
            ),
            (
                Boundary::BackendStructural,
                Mutation::UnauthenticatedRepresentation,
            ),
        ]),

        Relation::LifecycleExit { .. } => BTreeSet::from([
            (
                Boundary::CompilerStatic,
                Mutation::RequiredLifecycleExitMissing,
            ),
            (
                Boundary::BackendStructural,
                Mutation::RequiredLifecycleExitMissing,
            ),
        ]),
    }
}

/// Whether one case holds a relation's conserved amounts as
/// commitments, restated.
///
/// Read straight off the case's own representation map rather than from
/// the compiler's decision, which is the whole point of an oracle. The
/// pilots fix one representation per family, so `any` and `all` agree
/// here; the production derivation refuses the case where they would
/// not, and a mixed case therefore never reaches this comparison.
fn oracle_committed_amounts(relation: &Relation, case: &ExecutionCaseId) -> bool {
    let Relation::AmountConservation {
        input_objects,
        output_objects,
        ..
    } = relation
    else {
        return false;
    };

    input_objects.iter().chain(output_objects).any(|object| {
        case.representations.get(object) == Some(&RepresentationMode::PrivateCommitted)
    })
}

/// The typed external evidence one relation depends on, restated.
fn oracle_external_evidence(
    declaration: &RelationDeclaration,
    case: &ExecutionCaseId,
) -> BTreeSet<ExternalEvidenceRequirement> {
    match &declaration.relation {
        Relation::SubstrateConservation { asset } => {
            BTreeSet::from([ExternalEvidenceRequirement::SubstrateConservation {
                operation: declaration.id.operation(),
                asset: *asset,
            }])
        }
        Relation::AmountConservation { asset, .. }
            if oracle_committed_amounts(&declaration.relation, case) =>
        {
            BTreeSet::from(
                [ExternalEvidenceRequirement::ConfidentialValueConservation {
                    operation: declaration.id.operation(),
                    asset: *asset,
                }],
            )
        }
        _ => BTreeSet::new(),
    }
}

/// The representation the case fixed for one relation's object.
fn oracle_representation(
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

/// What one boundary answers about, and who answers it, restated.
///
/// One subject per boundary, except the external one, which answers once
/// per typed evidence requirement.
fn oracle_subjects(
    boundary: CoverageBoundary,
    external: &BTreeSet<ExternalEvidenceRequirement>,
) -> Vec<(EvidenceRole, Option<ExternalEvidenceRequirement>)> {
    match boundary {
        CoverageBoundary::CompilerStatic => vec![(EvidenceRole::CompilerAnalysisResult, None)],
        CoverageBoundary::BackendStructural => vec![(EvidenceRole::EmittedStructure, None)],
        CoverageBoundary::RuntimeCarrier => vec![(EvidenceRole::TargetExecution, None)],
        CoverageBoundary::ExternalEvidence => external
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

/// The positive purpose of one boundary and subject, restated.
fn oracle_positive_purpose(
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
fn oracle_negative_purpose(
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

/// The collateral one negative at one boundary expects, restated.
///
/// A runtime mutation blocks the active relations that depend on the
/// mutated one, so their closure is claimed; nothing at the other
/// boundaries proves isolation, so additional collateral is reported.
const fn oracle_collateral_policy(boundary: CoverageBoundary) -> CollateralPolicy {
    match boundary {
        CoverageBoundary::RuntimeCarrier => CollateralPolicy::RequireIntendedAndDependencyClosure,
        CoverageBoundary::CompilerStatic
        | CoverageBoundary::BackendStructural
        | CoverageBoundary::ExternalEvidence => CollateralPolicy::ReportAdditional,
    }
}

/// What one boundary's accepted projection compares, restated.
fn oracle_projection_subject(
    boundary: CoverageBoundary,
    external: &BTreeSet<ExternalEvidenceRequirement>,
) -> Option<ProjectionSubject> {
    match boundary {
        CoverageBoundary::RuntimeCarrier => Some(ProjectionSubject::RuntimeRelationVerdict),
        CoverageBoundary::CompilerStatic => Some(ProjectionSubject::CompilerSelectionResult),
        CoverageBoundary::BackendStructural => Some(ProjectionSubject::EmittedStructuralFact),
        CoverageBoundary::ExternalEvidence => {
            if external.is_empty() {
                None
            } else {
                Some(ProjectionSubject::ExternalReportSubjects {
                    requirements: external.clone(),
                })
            }
        }
    }
}

/// What one selected carrier depends on, restated.
///
/// A coordinator needs a canonical anchor; a complete-family proof needs
/// the census that makes completeness real; a routed source row needs
/// the routing, and a sponsor-confined one needs the region isolated
/// rather than the value exported.
fn oracle_selected_layout(
    analysis: &CarrierEligibility,
    entry: &EligibleCarrier,
) -> BTreeSet<LayoutRequirement> {
    use crate::carrier::CarrierAvailability;

    let mut requirements = BTreeSet::new();
    let relation = analysis.relation.clone();

    if matches!(
        entry.carrier,
        CarrierRole::OperationGlobal { .. } | CarrierRole::InputFamilyCoordinator { .. }
    ) {
        requirements.insert(LayoutRequirement::CanonicalCoordinator {
            operation: relation.operation(),
            anchor: entry.carrier.clone(),
        });
    }

    if entry.quantification == CarrierQuantification::CompleteFamilyProof
        && let crate::placement::SemanticScope::MemberLocal { side, object } = analysis.scope
    {
        requirements.insert(LayoutRequirement::AuthenticateFamilyCensus {
            relation: relation.clone(),
            side,
            object,
        });
        requirements.insert(LayoutRequirement::CompleteAndDisjointFamilies {
            relation: relation.clone(),
            side,
        });
    }

    for routing in &entry.routings {
        match routing.availability {
            CarrierAvailability::Intrinsic => {}
            CarrierAvailability::LayoutProvided => {
                requirements.insert(LayoutRequirement::MakeSourceAvailable {
                    relation: relation.clone(),
                    case: analysis.case.clone(),
                    carrier: entry.carrier.clone(),
                    source: routing.source.clone(),
                });
            }
            CarrierAvailability::SponsorRegionConfined => {
                requirements.insert(LayoutRequirement::IsolateSponsorRegion {
                    relation: relation.clone(),
                    case: analysis.case.clone(),
                });
            }
        }
    }

    requirements
}

/// The assignment alternatives of every relation-case, restated.
///
/// A raw scan over the feasible placements: one alternative per distinct
/// carrier selection, with exactly the layout that selection depends on.
/// The combined whole-transaction product never reaches the result,
/// because the alternatives are a set keyed by relation-case.
fn oracle_alternatives(
    eligibility: &[CarrierEligibility],
    placements: &[PlacementCandidate],
) -> BTreeMap<RelationCaseKey, BTreeSet<CarrierAssignmentAlternative>> {
    let mut alternatives: BTreeMap<RelationCaseKey, BTreeSet<CarrierAssignmentAlternative>> =
        BTreeMap::new();

    for placement in placements {
        for assignment in &placement.assignments {
            let key = assignment.key();
            let Some(analysis) = eligibility
                .iter()
                .find(|entry| entry.relation == key.relation && entry.case == key.case)
            else {
                continue;
            };
            let mut layout = BTreeSet::new();

            for carrier in &assignment.carriers {
                let entry = analysis.eligible.iter().find(|entry| {
                    entry.carrier == carrier.carrier
                        && entry.quantification == carrier.quantification
                });

                if let Some(entry) = entry {
                    layout.extend(oracle_selected_layout(analysis, entry));
                }
            }

            alternatives
                .entry(key)
                .or_default()
                .insert(CarrierAssignmentAlternative {
                    carriers: assignment.carriers.iter().cloned().collect(),
                    layout,
                });
        }
    }

    alternatives
}

/// The placed inputs one coverage analysis was derived from.
///
/// Absent for a plan-level analysis, which states no carrier obligation
/// and no accepted projection because it never saw a placement.
struct OraclePlacement<'a> {
    plans: &'a [RelationCasePlan],
    eligibility: &'a [CarrierEligibility],
    placements: &'a [PlacementCandidate],
}

/// The complete expected coverage of one relation in one case.
#[allow(clippy::too_many_lines)]
fn oracle_relation_projection(
    declaration: &RelationDeclaration,
    case: &ExecutionCaseId,
    placed: Option<&OraclePlacement<'_>>,
) -> RelationCoverageProjection {
    let relation = &declaration.relation;
    let key = RelationCaseKey {
        relation: declaration.id.clone(),
        case: case.clone(),
    };
    let activity = oracle_activity(relation, case);
    let boundaries = oracle_boundaries(relation, case);
    let external = oracle_external_evidence(declaration, case);
    let representation = oracle_representation(relation, case);
    let mutations = oracle_mutations(relation, case);
    let operands = relation_operands(declaration).expect("the pilot operand census derives");

    let id = |boundary: CoverageBoundary, purpose: CoveragePurpose| CoverageRequirementId {
        relation: declaration.id.clone(),
        case: case.clone(),
        boundary,
        purpose,
    };

    let mut positive = BTreeSet::new();
    let mut negative = BTreeSet::new();

    match activity {
        // One obligation and no mutation: a target rejecting a branch
        // this case never exercises would prove nothing about it.
        RelationActivity::Vacuous => {
            for boundary in &boundaries {
                positive.insert(PositiveCoverageRequirement {
                    id: id(*boundary, CoveragePurpose::InactiveAccept),
                    role: EvidenceRole::InactiveCaseAcceptance,
                    representation,
                    operands: Vec::new(),
                });
            }
        }

        RelationActivity::Active => {
            for boundary in &boundaries {
                for (role, subject) in oracle_subjects(*boundary, &external) {
                    positive.insert(PositiveCoverageRequirement {
                        id: id(
                            *boundary,
                            oracle_positive_purpose(*boundary, subject.as_ref()),
                        ),
                        role: role.clone(),
                        representation,
                        operands: operands.clone(),
                    });

                    for (owner, mutation) in &mutations {
                        if owner != boundary {
                            continue;
                        }

                        negative.insert(NegativeCoverageRequirement {
                            id: id(
                                *boundary,
                                oracle_negative_purpose(*boundary, subject.as_ref(), *mutation),
                            ),
                            role: role.clone(),
                            mutation: *mutation,
                            collateral: CollateralRequirement {
                                policy: oracle_collateral_policy(*boundary),
                                dependency_closure: BTreeSet::new(),
                            },
                        });
                    }
                }
            }
        }
    }

    let mut carrier = BTreeSet::new();
    let mut projections = BTreeMap::new();

    if let Some(placed) = placed {
        let alternatives = oracle_alternatives(placed.eligibility, placed.placements);

        if let Some(assignments) = alternatives.get(&key) {
            carrier = assignments.clone();

            for placed_carrier in assignments
                .iter()
                .flat_map(|alternative| alternative.carriers.iter())
            {
                positive.insert(PositiveCoverageRequirement {
                    id: id(
                        CoverageBoundary::RuntimeCarrier,
                        CoveragePurpose::CarrierExecution(placed_carrier.clone()),
                    ),
                    role: EvidenceRole::TargetExecution,
                    representation: None,
                    operands: Vec::new(),
                });
            }
        }

        if activity == RelationActivity::Active {
            // The runtime source rows the relation-case plan routes to a
            // carrier. Only the runtime boundary has any: a static,
            // structural, or external obligation is not answered by a
            // fact routed to a carrier.
            let sources = placed
                .plans
                .iter()
                .find(|plan| plan.relation == key.relation && plan.case == key.case)
                .map(|plan| {
                    let mut sources = plan
                        .runtime_requirements
                        .iter()
                        .flat_map(|requirement| requirement.sources.iter().cloned())
                        .collect::<Vec<SourceRequirement>>();
                    sources.sort();
                    sources.dedup();
                    sources
                })
                .unwrap_or_default();

            for boundary in &boundaries {
                let Some(subject) = oracle_projection_subject(*boundary, &external) else {
                    continue;
                };
                let role = oracle_subjects(*boundary, &external)
                    .first()
                    .map(|(role, _)| role.clone())
                    .expect("an answered boundary names its evidence role");

                projections.insert(
                    *boundary,
                    SemanticProjectionRequirement {
                        relation: key.relation.clone(),
                        case: key.case.clone(),
                        boundary: *boundary,
                        compare_relation_verdict: subject
                            == ProjectionSubject::RuntimeRelationVerdict,
                        subject,
                        operands: operands.clone(),
                        sources: if *boundary == CoverageBoundary::RuntimeCarrier {
                            sources.clone()
                        } else {
                            Vec::new()
                        },
                    },
                );

                positive.insert(PositiveCoverageRequirement {
                    id: id(*boundary, CoveragePurpose::AcceptedProjection),
                    role,
                    representation: None,
                    operands: operands.clone(),
                });
            }
        }
    }

    RelationCoverageProjection {
        activity,
        boundaries,
        positive,
        negative,
        carrier,
        projections,
        external_evidence: external,
    }
}

/// Every relation declaration in compiler scope, by stable identity.
fn oracle_declarations(
    relations: &CompilerRelationAnalysis,
) -> BTreeMap<RelationId, RelationDeclaration> {
    relations
        .graph
        .node_weights()
        .map(|node| (node.source.id.clone(), node.source.clone()))
        .collect()
}

/// The sponsor cases one operation admits, restated from its typed
/// open-flow relation — never from the operation's name.
fn oracle_sponsor_cases(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    operation: OperationId,
) -> Vec<SponsorCase> {
    let sponsored = declarations.values().any(|declaration| {
        declaration.id.operation() == operation
            && matches!(
                &declaration.relation,
                Relation::OpenFlowPolicy { allowed } if allowed.contains(&OpenFlowKind::FeeSponsor)
            )
    });

    if sponsored {
        vec![SponsorCase::Absent, SponsorCase::Present]
    } else {
        vec![SponsorCase::Absent]
    }
}

/// The execution cases of one candidate, derived directly.
///
/// The candidate already fixed every representation, so the only
/// dimension expanded here is the operation's own optional sponsor flow.
fn oracle_case_ids(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    candidate: &ProofPlanCandidate,
) -> BTreeSet<ExecutionCaseId> {
    let operations = declarations
        .keys()
        .map(RelationId::operation)
        .collect::<BTreeSet<_>>();
    let mut cases = BTreeSet::new();

    for operation in operations {
        let representations = candidate
            .representations
            .iter()
            .filter(|(choice, _)| choice.operation == operation)
            .map(|(choice, mode)| (choice.object, *mode))
            .collect::<BTreeMap<_, _>>();

        for sponsor in oracle_sponsor_cases(declarations, operation) {
            cases.insert(ExecutionCaseId {
                operation,
                sponsor,
                representations: representations.clone(),
            });
        }
    }

    cases
}

/// The expected coverage census: every in-scope relation crossed with
/// every applicable case of its own operation, exactly once.
fn oracle_census(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    cases: &BTreeSet<ExecutionCaseId>,
) -> BTreeSet<RelationCaseKey> {
    let mut census = BTreeSet::new();

    for relation in declarations.keys() {
        for case in cases {
            if relation.operation() != case.operation {
                continue;
            }

            census.insert(RelationCaseKey {
                relation: relation.clone(),
                case: case.clone(),
            });
        }
    }

    census
}

/// The complete expected coverage projection of one candidate.
fn oracle_projection(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    cases: &BTreeSet<ExecutionCaseId>,
    placed: Option<&OraclePlacement<'_>>,
) -> CoverageProjection {
    let requirements = oracle_census(declarations, cases)
        .into_iter()
        .map(|key| {
            let declaration = declarations
                .get(&key.relation)
                .expect("the census names declared relations only");
            let projection = oracle_relation_projection(declaration, &key.case, placed);

            (key, projection)
        })
        .collect();

    CoverageProjection { requirements }
}

// --- pilot fixtures ---

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

/// One pilot placed end to end, with its coverage and its oracle inputs.
struct Pilot {
    operation: OperationId,
    relations: CompilerRelationAnalysis,
    declarations: BTreeMap<RelationId, RelationDeclaration>,
    candidates: Vec<ProofPlanCandidate>,
    placed: Vec<PlacedProofPlanCandidate>,
    coverage: Vec<PlanCoverageAnalysis>,
}

fn pilot(operation: OperationId) -> Pilot {
    let input = bound_input(&[operation]);
    let relations = crate::relation::build_relation_analysis(&input).expect("relations");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("feasible plans")
        .candidates;
    let placed = place_feasible_proof_plans(&relations, &candidates, limits())
        .expect("placement analysis")
        .placed;

    assert_ne!(placed, [] as [PlacedProofPlanCandidate; 0]);

    let coverage = placed
        .iter()
        .map(|entry| analyze_placed_coverage(&relations, entry).expect("placed coverage"))
        .collect();
    let declarations = oracle_declarations(&relations);

    Pilot {
        operation,
        relations,
        declarations,
        candidates,
        placed,
        coverage,
    }
}

fn pilots() -> [Pilot; 2] {
    [
        pilot(OperationId::CompactAsh),
        pilot(OperationId::TransferLive),
    ]
}

impl Pilot {
    /// The carrier eligibility of one placed candidate.
    fn eligibility(&self, entry: &PlacedProofPlanCandidate) -> Vec<CarrierEligibility> {
        crate::carrier::relation_case_eligibility(&self.relations, &entry.relation_case_plans)
            .expect("eligibility")
    }

    /// The oracle's expected projection of one placed candidate.
    fn expected(
        &self,
        candidate: &ProofPlanCandidate,
        entry: &PlacedProofPlanCandidate,
        eligibility: &[CarrierEligibility],
    ) -> CoverageProjection {
        let cases = oracle_case_ids(&self.declarations, candidate);
        let placed = OraclePlacement {
            plans: &entry.relation_case_plans,
            eligibility,
            placements: &entry.feasible_placements,
        };

        oracle_projection(&self.declarations, &cases, Some(&placed))
    }

    /// Each placed candidate with its production coverage and the
    /// oracle's independent expectation.
    fn compared(
        &self,
    ) -> Vec<(
        &PlacedProofPlanCandidate,
        &PlanCoverageAnalysis,
        CoverageProjection,
    )> {
        self.placed
            .iter()
            .zip(self.coverage.iter())
            .map(|(entry, coverage)| {
                let eligibility = self.eligibility(entry);
                let expected = self.expected(&entry.proof_plan, entry, &eligibility);

                (entry, coverage, expected)
            })
            .collect()
    }
}

/// Production and the oracle agree on one complete projection.
///
/// Compared component by component before the whole value, so a
/// disagreement names the relation-case and the requirement class rather
/// than printing two complete censuses.
fn agree(operation: OperationId, production: &CoverageProjection, expected: &CoverageProjection) {
    assert_eq!(
        production.requirements.keys().collect::<Vec<_>>(),
        expected.requirements.keys().collect::<Vec<_>>(),
        "{operation:?} relation-case census",
    );

    for (key, produced) in &production.requirements {
        let oracle = &expected.requirements[key];

        assert_eq!(produced.activity, oracle.activity, "{operation:?} {key:?}");
        assert_eq!(
            produced.boundaries, oracle.boundaries,
            "{operation:?} {key:?}",
        );
        assert_eq!(
            produced.external_evidence, oracle.external_evidence,
            "{operation:?} {key:?}",
        );

        for (class, produced, oracle) in [
            (
                "positive",
                produced
                    .positive
                    .iter()
                    .map(|entry| format!("{entry:?}"))
                    .collect::<BTreeSet<_>>(),
                oracle
                    .positive
                    .iter()
                    .map(|entry| format!("{entry:?}"))
                    .collect::<BTreeSet<_>>(),
            ),
            (
                "negative",
                produced
                    .negative
                    .iter()
                    .map(|entry| format!("{entry:?}"))
                    .collect::<BTreeSet<_>>(),
                oracle
                    .negative
                    .iter()
                    .map(|entry| format!("{entry:?}"))
                    .collect::<BTreeSet<_>>(),
            ),
            (
                "carrier",
                produced
                    .carrier
                    .iter()
                    .map(|entry| format!("{entry:?}"))
                    .collect::<BTreeSet<_>>(),
                oracle
                    .carrier
                    .iter()
                    .map(|entry| format!("{entry:?}"))
                    .collect::<BTreeSet<_>>(),
            ),
        ] {
            let unexpected = produced.difference(&oracle).collect::<Vec<_>>();
            let missing = oracle.difference(&produced).collect::<Vec<_>>();

            assert!(
                unexpected.is_empty() && missing.is_empty(),
                "{operation:?} {key:?} {class}: production has {unexpected:?}, oracle has {missing:?}",
            );
        }

        assert_eq!(
            produced.projections, oracle.projections,
            "{operation:?} {key:?}",
        );
    }

    assert_eq!(production, expected, "{operation:?}");
}

// --- §12.5 complete pilot relation × case matrix ---

#[test]
fn the_oracle_reproduces_the_complete_pilot_relation_case_matrix() {
    for pilot in pilots() {
        let mut compared = 0_usize;

        for (_, coverage, expected) in pilot.compared() {
            compared += 1;
            agree(pilot.operation, &coverage.project(), &expected);
        }

        assert!(compared > 0, "{:?}", pilot.operation);
    }
}

#[test]
fn the_oracle_reproduces_the_plan_level_matrix_before_any_placement() {
    for pilot in pilots() {
        for candidate in &pilot.candidates {
            let cases = crate::case::execution_cases(&pilot.relations, candidate).expect("cases");
            let plans = classify_relation_cases(&pilot.relations, &cases).expect("classification");
            let production = analyze_plan_coverage(&pilot.relations, &plans).expect("coverage");
            let expected = oracle_projection(
                &pilot.declarations,
                &oracle_case_ids(&pilot.declarations, candidate),
                None,
            );

            assert_eq!(production.project(), expected, "{:?}", pilot.operation);
        }
    }
}

#[test]
fn the_oracle_census_is_the_exact_relation_case_product() {
    for pilot in pilots() {
        for (entry, coverage, _) in pilot.compared() {
            let cases = oracle_case_ids(&pilot.declarations, &entry.proof_plan);
            let census = oracle_census(&pilot.declarations, &cases);

            assert_eq!(coverage.keys(), census, "{:?}", pilot.operation);
            assert_eq!(
                census.len(),
                pilot.declarations.len() * cases.len(),
                "{:?}",
                pilot.operation,
            );
        }
    }
}

#[test]
fn the_oracle_expects_sponsor_cardinality_in_both_pilots() {
    // The wave-0 repair, restated as an independent expectation: both
    // pilots declare a sponsor-family cardinality on each side, it is
    // inactive without a sponsor region and focused-rejected with one,
    // and its minimum of zero states no below-minimum class.
    for pilot in pilots() {
        for side in [TransactionSide::Input, TransactionSide::Output] {
            let relation = RelationId::new(
                pilot.operation,
                RelationKind::Cardinality,
                RelationSubject::ObjectFamily {
                    side,
                    object: ObjectId::PlainLbtc,
                },
            );
            let declaration = pilot
                .declarations
                .get(&relation)
                .unwrap_or_else(|| panic!("{:?} declares {relation:?}", pilot.operation));
            // A cardinality relation conserves nothing, so the case's
            // representations cannot change its mutations; the empty
            // map states that rather than borrowing a pilot case whose
            // representation would read as load-bearing here.
            let mutations = oracle_mutations(
                &declaration.relation,
                &ExecutionCaseId {
                    operation: pilot.operation,
                    sponsor: SponsorCase::Present,
                    representations: BTreeMap::new(),
                },
            );

            assert!(!mutations.contains(&(
                CoverageBoundary::RuntimeCarrier,
                RelationMutation::CardinalityBelowMinimum,
            )));
            assert!(mutations.iter().any(|(boundary, mutation)| {
                *boundary == CoverageBoundary::RuntimeCarrier
                    && matches!(mutation, RelationMutation::CardinalityAboveMaximum { .. })
            }));

            let mut dispositions = BTreeMap::new();

            for (_, coverage, _) in pilot.compared() {
                for plan in coverage.plans().filter(|plan| plan.relation == relation) {
                    dispositions.insert(plan.case.sponsor, plan.activity);
                }
            }

            assert_eq!(
                dispositions,
                BTreeMap::from([
                    (SponsorCase::Absent, RelationActivity::Vacuous),
                    (SponsorCase::Present, RelationActivity::Active),
                ]),
                "{:?} {side:?}",
                pilot.operation,
            );
        }
    }
}

// --- §12.3 the independent collateral oracle ---

/// The relation prerequisite edges of one coverage census, restated.
///
/// Prerequisite → dependent, restricted to one operation and one case,
/// with both endpoints in the census: a prerequisite holding in the
/// sponsored case says nothing about the sponsorless one, and an edge
/// crossing two operations is a claim neither analysis owns.
fn oracle_prerequisites(
    relations: &CompilerRelationAnalysis,
    census: &BTreeSet<RelationCaseKey>,
) -> BTreeSet<(RelationCaseKey, RelationCaseKey)> {
    let mut edges = BTreeSet::new();

    for edge in relations.project().edges {
        let AnalysisNodeId::SourceRelation(prerequisite) = edge.source;
        let AnalysisNodeId::SourceRelation(dependent) = edge.target;

        if prerequisite.operation() != dependent.operation() {
            continue;
        }

        for key in census {
            let source = RelationCaseKey {
                relation: prerequisite.clone(),
                case: key.case.clone(),
            };
            let target = RelationCaseKey {
                relation: dependent.clone(),
                case: key.case.clone(),
            };

            if census.contains(&source) && census.contains(&target) {
                edges.insert((source, target));
            }
        }
    }

    edges
}

/// The repeated-complete-scan closure of one relation-case.
///
/// The §12.3 fixpoint: start with the intended relation, repeatedly add
/// every dependent whose prerequisite is already present, stop when a
/// complete scan adds nothing. The intended relation is required in its
/// own right and is never its own collateral, and an inactive dependent
/// is traversed but never claimed — a relation that never activates in
/// the case cannot be observed to become blocked.
fn oracle_closure(
    edges: &BTreeSet<(RelationCaseKey, RelationCaseKey)>,
    activity: &BTreeMap<RelationCaseKey, RelationActivity>,
    start: &RelationCaseKey,
) -> BTreeSet<RelationCaseKey> {
    let mut reached = BTreeSet::from([start.clone()]);

    loop {
        let mut added = false;

        for (prerequisite, dependent) in edges {
            if reached.contains(prerequisite) && !reached.contains(dependent) {
                reached.insert(dependent.clone());
                added = true;
            }
        }

        if !added {
            break;
        }
    }

    reached
        .into_iter()
        .filter(|key| key != start)
        .filter(|key| activity.get(key) == Some(&RelationActivity::Active))
        .collect()
}

/// The activity of every covered relation-case.
fn oracle_activities(
    declarations: &BTreeMap<RelationId, RelationDeclaration>,
    census: &BTreeSet<RelationCaseKey>,
) -> BTreeMap<RelationCaseKey, RelationActivity> {
    census
        .iter()
        .map(|key| {
            let declaration = declarations
                .get(&key.relation)
                .expect("the census names declared relations only");

            (
                key.clone(),
                oracle_activity(&declaration.relation, &key.case),
            )
        })
        .collect()
}

#[test]
fn the_repeated_scan_closure_equals_the_bound_dependency_collateral() {
    for pilot in pilots() {
        let mut claimed = 0_usize;

        for (entry, coverage, _) in pilot.compared() {
            let census = oracle_census(
                &pilot.declarations,
                &oracle_case_ids(&pilot.declarations, &entry.proof_plan),
            );
            let edges = oracle_prerequisites(&pilot.relations, &census);
            let activity = oracle_activities(&pilot.declarations, &census);
            let mut bound = coverage.clone();

            resolve_coverage_dependencies(&mut bound, &pilot.relations).expect("resolution");

            for plan in bound.plans() {
                let expected = oracle_closure(&edges, &activity, &plan.key());

                for requirement in &plan.negative {
                    match requirement.collateral.policy {
                        CollateralPolicy::RequireIntendedAndDependencyClosure => {
                            assert_eq!(
                                requirement.collateral.dependency_closure, expected,
                                "{:?} {:?}",
                                pilot.operation, plan.relation,
                            );

                            if !expected.is_empty() {
                                claimed += 1;
                            }
                        }
                        CollateralPolicy::ReportAdditional => {
                            assert!(requirement.collateral.dependency_closure.is_empty());
                        }
                    }
                }
            }
        }

        assert!(claimed > 0, "{:?}", pilot.operation);
    }
}

#[test]
fn the_independent_prerequisite_edges_equal_the_graph_projection() {
    for pilot in pilots() {
        for (entry, coverage, _) in pilot.compared() {
            let census = oracle_census(
                &pilot.declarations,
                &oracle_case_ids(&pilot.declarations, &entry.proof_plan),
            );
            let expected = oracle_prerequisites(&pilot.relations, &census);
            let graph =
                analyze_coverage_dependencies(coverage, &pilot.relations).expect("resolution");
            let produced = graph
                .project()
                .edges
                .into_iter()
                .filter(|dependency| dependency.edge == CoverageEdge::RelationPrerequisite)
                .map(|dependency| {
                    (
                        dependency
                            .source
                            .relation_case()
                            .expect("a prerequisite edge joins two relation-cases")
                            .clone(),
                        dependency
                            .target
                            .relation_case()
                            .expect("a prerequisite edge joins two relation-cases")
                            .clone(),
                    )
                })
                .collect::<BTreeSet<_>>();

            assert_eq!(produced, expected, "{:?}", pilot.operation);
            assert!(!expected.is_empty(), "{:?}", pilot.operation);
        }
    }
}

// --- §12.4 the independent SCC oracle ---

/// Every cyclic component by mutual reachability, canonically ordered.
///
/// u and v are equivalent exactly when u leads to v and v leads to u. A
/// singleton component is cyclic only through its own self-loop. Test
/// only: the transitive closure is recomputed by repeated scan, which
/// need not scale.
fn oracle_components(
    nodes: &[CoverageNodeId],
    edges: &[CoverageDependency],
) -> Vec<CoverageCycleComponent> {
    let mut reachable = BTreeSet::new();

    for edge in edges {
        reachable.insert((edge.source.clone(), edge.target.clone()));
    }

    loop {
        let mut added = false;
        let snapshot = reachable.clone();

        for (source, middle) in &snapshot {
            for (other, target) in &snapshot {
                if middle == other && reachable.insert((source.clone(), target.clone())) {
                    added = true;
                }
            }
        }

        if !added {
            break;
        }
    }

    let mut assigned = BTreeSet::new();
    let mut components = Vec::new();

    for node in nodes {
        if assigned.contains(node) {
            continue;
        }

        let mut members = nodes
            .iter()
            .filter(|other| {
                *other == node
                    || (reachable.contains(&(node.clone(), (*other).clone()))
                        && reachable.contains(&((*other).clone(), node.clone())))
            })
            .cloned()
            .collect::<Vec<_>>();
        members.sort();
        members.dedup();

        for member in &members {
            assigned.insert(member.clone());
        }

        let cyclic = members.len() > 1 || reachable.contains(&(node.clone(), node.clone()));

        if !cyclic {
            continue;
        }

        let mut internal_edges = edges
            .iter()
            .filter(|edge| members.contains(&edge.source) && members.contains(&edge.target))
            .cloned()
            .collect::<Vec<_>>();
        internal_edges.sort();

        components.push(CoverageCycleComponent {
            members,
            internal_edges,
        });
    }

    components.sort();
    components
}

// --- synthetic graph fixtures ---

fn synthetic_case(operation: OperationId) -> ExecutionCaseId {
    ExecutionCaseId {
        operation,
        sponsor: SponsorCase::Absent,
        representations: BTreeMap::new(),
    }
}

fn synthetic_key(operation: OperationId, kind: RelationKind) -> RelationCaseKey {
    RelationCaseKey {
        relation: RelationId::new(operation, kind, RelationSubject::Operation),
        case: synthetic_case(operation),
    }
}

fn synthetic_keys(operation: OperationId) -> [RelationCaseKey; 4] {
    [
        synthetic_key(operation, RelationKind::Cardinality),
        synthetic_key(operation, RelationKind::Recognition),
        synthetic_key(operation, RelationKind::Authorization),
        synthetic_key(operation, RelationKind::Conservation),
    ]
}

fn synthetic_node(key: &RelationCaseKey, activity: RelationActivity) -> CoverageNode {
    CoverageNode {
        id: CoverageNodeId::RelationCase(key.clone()),
        activity: Some(activity),
    }
}

fn prerequisite(source: &RelationCaseKey, target: &RelationCaseKey) -> CoverageDependency {
    CoverageDependency {
        source: CoverageNodeId::RelationCase(source.clone()),
        target: CoverageNodeId::RelationCase(target.clone()),
        edge: CoverageEdge::RelationPrerequisite,
    }
}

fn synthetic_census(nodes: &[CoverageNode]) -> CoverageDefinitionCensus {
    let mut census = CoverageDefinitionCensus::new();

    for node in nodes {
        census.define_once(node.clone()).expect("distinct symbol");
    }

    census
}

fn synthetic_graph(
    nodes: &[CoverageNode],
    edges: &[CoverageDependency],
) -> Result<CoverageDependencyGraph, CompileError> {
    build_coverage_graph(&synthetic_census(nodes), edges)
}

fn active_nodes(keys: &[RelationCaseKey]) -> Vec<CoverageNode> {
    keys.iter()
        .map(|key| synthetic_node(key, RelationActivity::Active))
        .collect()
}

/// The edge set of a synthetic graph, as the closure oracle reads it.
fn synthetic_edge_pairs(
    edges: &[CoverageDependency],
) -> BTreeSet<(RelationCaseKey, RelationCaseKey)> {
    edges
        .iter()
        .filter(|edge| edge.edge == CoverageEdge::RelationPrerequisite)
        .map(|edge| {
            (
                edge.source.relation_case().expect("relation-case").clone(),
                edge.target.relation_case().expect("relation-case").clone(),
            )
        })
        .collect()
}

fn all_active(keys: &[RelationCaseKey]) -> BTreeMap<RelationCaseKey, RelationActivity> {
    keys.iter()
        .map(|key| (key.clone(), RelationActivity::Active))
        .collect()
}

// --- §12.5 dependency shapes ---

#[test]
fn the_closure_oracle_agrees_on_a_dependency_chain() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let edges = vec![
        prerequisite(&keys[0], &keys[1]),
        prerequisite(&keys[1], &keys[2]),
        prerequisite(&keys[2], &keys[3]),
    ];
    let graph = synthetic_graph(&active_nodes(&keys), &edges).expect("chain");
    let pairs = synthetic_edge_pairs(&edges);
    let activity = all_active(&keys);

    assert_eq!(graph.validate_acyclic(), Ok(()));
    assert_eq!(
        oracle_components(&graph_nodes(&keys), &edges),
        [] as [CoverageCycleComponent; 0]
    );

    for key in &keys {
        assert_eq!(
            graph.dependency_collateral(key),
            oracle_closure(&pairs, &activity, key),
        );
    }

    // The chain really is transitive, so the comparison is not vacuous.
    assert_eq!(
        oracle_closure(&pairs, &activity, &keys[0]),
        BTreeSet::from([keys[1].clone(), keys[2].clone(), keys[3].clone()]),
    );
}

#[test]
fn the_closure_oracle_agrees_on_a_dependency_diamond() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let edges = vec![
        prerequisite(&keys[0], &keys[1]),
        prerequisite(&keys[0], &keys[2]),
        prerequisite(&keys[1], &keys[3]),
        prerequisite(&keys[2], &keys[3]),
    ];
    let graph = synthetic_graph(&active_nodes(&keys), &edges).expect("diamond");
    let pairs = synthetic_edge_pairs(&edges);
    let activity = all_active(&keys);

    assert_eq!(graph.validate_acyclic(), Ok(()));

    for key in &keys {
        assert_eq!(
            graph.dependency_collateral(key),
            oracle_closure(&pairs, &activity, key),
        );
    }

    // The join is claimed once, not once per path.
    assert_eq!(
        oracle_closure(&pairs, &activity, &keys[0]),
        BTreeSet::from([keys[1].clone(), keys[2].clone(), keys[3].clone()]),
    );
}

#[test]
fn the_closure_oracle_never_claims_an_inactive_dependent() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let nodes = vec![
        synthetic_node(&keys[0], RelationActivity::Active),
        synthetic_node(&keys[1], RelationActivity::Vacuous),
        synthetic_node(&keys[2], RelationActivity::Active),
        synthetic_node(&keys[3], RelationActivity::Active),
    ];
    let edges = vec![
        prerequisite(&keys[0], &keys[1]),
        prerequisite(&keys[1], &keys[2]),
        prerequisite(&keys[0], &keys[3]),
    ];
    let graph = synthetic_graph(&nodes, &edges).expect("graph");
    let pairs = synthetic_edge_pairs(&edges);
    let activity = BTreeMap::from([
        (keys[0].clone(), RelationActivity::Active),
        (keys[1].clone(), RelationActivity::Vacuous),
        (keys[2].clone(), RelationActivity::Active),
        (keys[3].clone(), RelationActivity::Active),
    ]);

    // Traversed through, never claimed.
    assert_eq!(
        oracle_closure(&pairs, &activity, &keys[0]),
        BTreeSet::from([keys[2].clone(), keys[3].clone()]),
    );
    assert_eq!(
        graph.dependency_collateral(&keys[0]),
        oracle_closure(&pairs, &activity, &keys[0]),
    );
}

fn graph_nodes(keys: &[RelationCaseKey]) -> Vec<CoverageNodeId> {
    keys.iter()
        .map(|key| CoverageNodeId::RelationCase(key.clone()))
        .collect()
}

#[test]
fn the_mutual_reachability_oracle_agrees_on_a_self_loop() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let edges = vec![prerequisite(&keys[0], &keys[0])];
    let graph = synthetic_graph(&active_nodes(&keys), &edges).expect("graph");
    let expected = oracle_components(&graph_nodes(&keys), &edges);

    assert_eq!(expected.len(), 1);
    assert_eq!(
        expected[0].members,
        vec![CoverageNodeId::RelationCase(keys[0].clone())],
    );
    assert_eq!(graph.cyclic_components(), expected);
    assert_eq!(
        graph.validate_acyclic(),
        Err(CompileError::CoverageDependencyCycle {
            components: expected,
        }),
    );
}

#[test]
fn the_mutual_reachability_oracle_agrees_on_a_multi_node_component() {
    let keys = synthetic_keys(OperationId::CompactAsh);
    let edges = vec![
        prerequisite(&keys[0], &keys[1]),
        prerequisite(&keys[1], &keys[2]),
        prerequisite(&keys[2], &keys[0]),
        // A dependent outside the cycle: reachable from it, never
        // mutually so, and therefore not a member.
        prerequisite(&keys[2], &keys[3]),
    ];
    let graph = synthetic_graph(&active_nodes(&keys), &edges).expect("graph");
    let expected = oracle_components(&graph_nodes(&keys), &edges);

    assert_eq!(expected.len(), 1);
    assert_eq!(expected[0].members.len(), 3);
    assert!(
        !expected[0]
            .members
            .contains(&CoverageNodeId::RelationCase(keys[3].clone()))
    );
    assert_eq!(graph.cyclic_components(), expected);

    // The diagnostic is canonical: a permuted declaration order produces
    // the same component.
    let mut permuted = edges;
    permuted.reverse();

    let second = synthetic_graph(&active_nodes(&keys), &permuted).expect("graph");

    assert_eq!(second.cyclic_components(), expected);
    assert_eq!(oracle_components(&graph_nodes(&keys), &permuted), expected);
}

#[test]
fn a_cross_operation_edge_is_rejected_and_the_oracle_expects_no_closure() {
    let compact = synthetic_key(OperationId::CompactAsh, RelationKind::Cardinality);
    let live = synthetic_key(OperationId::TransferLive, RelationKind::Cardinality);
    let nodes = vec![
        synthetic_node(&compact, RelationActivity::Active),
        synthetic_node(&live, RelationActivity::Active),
    ];
    let edges = vec![prerequisite(&compact, &live)];

    // The oracle restricts every prerequisite to one operation, so it
    // expects no edge and no collateral at all.
    assert!(
        oracle_closure(
            &BTreeSet::new(),
            &all_active(&[compact.clone(), live.clone()]),
            &compact
        )
        .is_empty()
    );
    assert_eq!(
        synthetic_graph(&nodes, &edges).map(|_| ()),
        Err(CompileError::CrossOperationCoverageDependency {
            prerequisite: Box::new(CoverageNodeId::RelationCase(compact)),
            dependent: Box::new(CoverageNodeId::RelationCase(live)),
        }),
    );
}

// --- §12.5 defect cases ---
//
// Each corrupts a derived analysis and asserts twice: production rejects
// it, and the oracle's independent expectation disagrees with it.

/// One placed pilot with everything a corruption test needs.
struct Corruptible {
    entry: PlacedProofPlanCandidate,
    eligibility: Vec<CarrierEligibility>,
    coverage: PlanCoverageAnalysis,
    expected: CoverageProjection,
}

fn corruptible() -> Corruptible {
    let pilot = pilot(OperationId::CompactAsh);
    let entry = pilot.placed.first().expect("a placed plan").clone();
    let coverage = pilot.coverage.first().expect("its coverage").clone();
    let eligibility = pilot.eligibility(&entry);
    let expected = pilot.expected(&entry.proof_plan, &entry, &eligibility);

    Corruptible {
        entry,
        eligibility,
        coverage,
        expected,
    }
}

impl Corruptible {
    fn revalidate(&self, coverage: &PlanCoverageAnalysis) -> Result<(), CompileError> {
        validate_placement_coverage(
            &self.eligibility,
            &self.entry.feasible_placements,
            &self.entry.layout_requirements,
            coverage,
        )
    }

    fn census(&self, coverage: &PlanCoverageAnalysis) -> Result<(), CompileError> {
        validate_coverage_census(&self.entry.relation_case_plans, coverage)
    }

    /// The oracle disagrees with a corrupted analysis.
    fn disagrees(&self, coverage: &PlanCoverageAnalysis) {
        assert_ne!(coverage.project(), self.expected);
    }

    fn key_where(&self, predicate: impl Fn(&RelationCoveragePlan) -> bool) -> RelationCaseKey {
        self.coverage
            .plans()
            .find(|plan| predicate(plan))
            .expect("a matching coverage plan")
            .key()
    }
}

fn plan_mut<'a>(
    coverage: &'a mut PlanCoverageAnalysis,
    key: &RelationCaseKey,
) -> &'a mut RelationCoveragePlan {
    coverage
        .operations
        .get_mut(&key.relation.operation())
        .expect("the operation is analyzed")
        .requirements
        .get_mut(key)
        .expect("the relation-case is covered")
}

#[test]
fn a_missing_coverage_entry_disagrees_with_the_oracle_and_is_rejected() {
    let subject = corruptible();
    let mut coverage = subject.coverage.clone();
    let key = subject.key_where(|_| true);

    coverage
        .operations
        .get_mut(&key.relation.operation())
        .expect("the operation is analyzed")
        .requirements
        .remove(&key);

    subject.disagrees(&coverage);
    assert_eq!(
        subject.census(&coverage),
        Err(CompileError::CoverageCensusMismatch {
            missing: vec![key],
            unexpected: Vec::new(),
        }),
    );
}

#[test]
fn a_duplicate_coverage_entry_disagrees_with_the_oracle_and_is_rejected() {
    let subject = corruptible();
    let mut coverage = subject.coverage.clone();
    let key = subject.key_where(|plan| !plan.positive.is_empty());
    let plan = plan_mut(&mut coverage, &key);
    let duplicate = plan.positive[0].clone();
    let id = duplicate.id.clone();

    plan.positive.push(duplicate);

    // The projection is set-shaped, so the duplicate alone does not
    // change it — the identity census is what rejects, and the oracle
    // states the same requirement exactly once.
    assert_eq!(coverage.project(), subject.expected);
    assert_eq!(
        subject.census(&coverage),
        Err(CompileError::DuplicateCoverageRequirement { requirement: id }),
    );
}

#[test]
fn an_unexpected_coverage_entry_disagrees_with_the_oracle_and_is_rejected() {
    let subject = corruptible();
    let mut coverage = subject.coverage.clone();
    let key = subject.key_where(|plan| plan.activity == RelationActivity::Active);
    let unexpected = CoverageRequirementId {
        relation: key.relation.clone(),
        case: key.case.clone(),
        boundary: CoverageBoundary::RuntimeCarrier,
        purpose: CoveragePurpose::FocusedReject(RelationMutation::WrongRootEffect),
    };

    plan_mut(&mut coverage, &key)
        .negative
        .push(NegativeCoverageRequirement {
            id: unexpected,
            role: EvidenceRole::TargetExecution,
            mutation: RelationMutation::WrongRootEffect,
            collateral: CollateralRequirement {
                policy: CollateralPolicy::RequireIntendedAndDependencyClosure,
                dependency_closure: BTreeSet::new(),
            },
        });

    // A mutation class the relation never requires: the census validator
    // accepts the shape, and only the oracle's restated catalogue knows
    // the class does not belong to this relation.
    subject.disagrees(&coverage);

    // The same relation-case dropped from the plan set is the census
    // form of an unexpected entry.
    let mut plans = subject.entry.relation_case_plans.clone();
    plans.retain(|plan| plan.relation != key.relation || plan.case != key.case);

    assert_eq!(
        validate_coverage_census(&plans, &subject.coverage),
        Err(CompileError::CoverageCensusMismatch {
            missing: Vec::new(),
            unexpected: vec![key],
        }),
    );
}

#[test]
fn an_active_relation_case_missing_its_positive_disagrees_with_the_oracle_and_is_rejected() {
    let subject = corruptible();
    let mut coverage = subject.coverage.clone();
    let key = subject.key_where(|plan| {
        plan.activity == RelationActivity::Active
            && plan.boundaries.contains(&CoverageBoundary::RuntimeCarrier)
    });

    plan_mut(&mut coverage, &key).positive.clear();

    subject.disagrees(&coverage);
    assert_eq!(
        subject.census(&coverage),
        Err(CompileError::MissingPositiveCoverage {
            relation: key.relation,
            case: key.case,
            boundary: CoverageBoundary::RuntimeCarrier,
        }),
    );
}

#[test]
fn an_active_relation_case_missing_its_negative_disagrees_with_the_oracle_and_is_rejected() {
    let subject = corruptible();
    let mut coverage = subject.coverage.clone();
    let key = subject.key_where(|plan| {
        plan.activity == RelationActivity::Active
            && plan.boundaries.contains(&CoverageBoundary::RuntimeCarrier)
    });

    plan_mut(&mut coverage, &key).negative.clear();

    subject.disagrees(&coverage);
    assert_eq!(
        subject.census(&coverage),
        Err(CompileError::MissingNegativeCoverage {
            relation: key.relation,
            case: key.case,
            boundary: CoverageBoundary::RuntimeCarrier,
        }),
    );
}

/// One synthetic carrier obligation, for the shapes a derived analysis
/// never produces.
fn synthetic_carrier() -> crate::coverage::CarrierCoverageRequirement {
    crate::coverage::CarrierCoverageRequirement {
        relation_case: synthetic_key(OperationId::CompactAsh, RelationKind::Cardinality),
        allowed_assignments: BTreeSet::from([CarrierAssignmentAlternative {
            carriers: BTreeSet::from([PlacedCarrier {
                carrier: CarrierRole::OperationGlobal {
                    operation: OperationId::CompactAsh,
                    anchor: ObjectId::Ash,
                },
                quantification: CarrierQuantification::Single,
            }]),
            layout: BTreeSet::new(),
        }]),
    }
}

#[test]
fn an_inactive_relation_case_with_a_runtime_carrier_disagrees_with_the_oracle_and_is_rejected() {
    let subject = corruptible();
    let mut coverage = subject.coverage.clone();
    let key = subject.key_where(|plan| plan.activity == RelationActivity::Vacuous);
    let mut carrier = synthetic_carrier();
    carrier.relation_case = key.clone();

    plan_mut(&mut coverage, &key).carrier = Some(carrier);

    subject.disagrees(&coverage);
    assert_eq!(
        subject.revalidate(&coverage),
        Err(CompileError::UnexpectedRuntimeCarrierCoverage {
            relation: key.relation,
            case: key.case,
        }),
    );
}

#[test]
fn a_runtime_relation_case_missing_its_carrier_disagrees_with_the_oracle_and_is_rejected() {
    let subject = corruptible();
    let mut coverage = subject.coverage.clone();
    let key = subject.key_where(|plan| plan.carrier.is_some());

    plan_mut(&mut coverage, &key).carrier = None;

    subject.disagrees(&coverage);
    assert_eq!(
        subject.revalidate(&coverage),
        Err(CompileError::MissingCarrierCoverage {
            relation: key.relation,
            case: key.case,
        }),
    );
}

#[test]
fn an_external_relation_case_with_a_runtime_carrier_disagrees_with_the_oracle_and_is_rejected() {
    let subject = corruptible();
    let mut coverage = subject.coverage.clone();
    let key = subject.key_where(|plan| {
        plan.boundaries
            .contains(&CoverageBoundary::ExternalEvidence)
            && plan.activity == RelationActivity::Active
    });
    let mut carrier = synthetic_carrier();
    carrier.relation_case = key.clone();

    // An externally evidenced obligation has no runtime carrier at all:
    // a report that does not exist yet cannot be answered by execution.
    plan_mut(&mut coverage, &key).carrier = Some(carrier);

    subject.disagrees(&coverage);
    assert_eq!(
        subject.revalidate(&coverage),
        Err(CompileError::UnexpectedRuntimeCarrierCoverage {
            relation: key.relation,
            case: key.case,
        }),
    );
}

#[test]
fn a_selected_carrier_missing_its_layout_reference_disagrees_with_the_oracle_and_is_rejected() {
    let subject = corruptible();
    let mut coverage = subject.coverage.clone();
    let key = subject.key_where(|plan| {
        plan.carrier.as_ref().is_some_and(|carrier| {
            carrier
                .allowed_assignments
                .iter()
                .any(|alternative| !alternative.layout.is_empty())
        })
    });
    let carrier = plan_mut(&mut coverage, &key)
        .carrier
        .as_mut()
        .expect("a carried relation-case");
    let dropped = carrier
        .allowed_assignments
        .iter()
        .find(|alternative| !alternative.layout.is_empty())
        .expect("an alternative with a layout dependency")
        .clone();
    let mut stripped = dropped.clone();
    stripped.layout.clear();

    carrier.allowed_assignments.remove(&dropped);
    carrier.allowed_assignments.insert(stripped);

    subject.disagrees(&coverage);
    assert_eq!(
        subject.revalidate(&coverage),
        Err(CompileError::MissingCoverageLayoutRequirement {
            relation: key.relation,
            case: key.case,
        }),
    );
}

#[test]
fn a_sponsor_amount_introduced_into_coverage_disagrees_with_the_oracle_and_is_rejected() {
    let subject = corruptible();
    let mut coverage = subject.coverage.clone();
    let key = subject.key_where(|plan| plan.carrier.is_some());
    let sponsor_amount = OperandId::new(
        key.relation.clone(),
        OperandRole::ObjectFamilyAmount {
            side: TransactionSide::Input,
            object: ObjectId::PlainLbtc,
        },
    );

    plan_mut(&mut coverage, &key)
        .positive
        .push(PositiveCoverageRequirement {
            id: CoverageRequirementId {
                relation: key.relation.clone(),
                case: key.case.clone(),
                boundary: CoverageBoundary::RuntimeCarrier,
                purpose: CoveragePurpose::ActiveAccept,
            },
            role: EvidenceRole::TargetExecution,
            representation: None,
            operands: vec![sponsor_amount],
        });

    subject.disagrees(&coverage);
    assert_eq!(
        subject.revalidate(&coverage),
        Err(CompileError::SponsorValueRead),
    );

    // And the oracle's own expectation never names one, in either pilot.
    for pilot in pilots() {
        for (_, _, expected) in pilot.compared() {
            for requirement in expected.requirements.values() {
                for operand in requirement
                    .positive
                    .iter()
                    .flat_map(|entry| entry.operands.iter())
                    .chain(
                        requirement
                            .projections
                            .values()
                            .flat_map(|entry| entry.operands.iter()),
                    )
                {
                    assert!(
                        !crate::source::is_sponsor_amount_operand(operand.role()),
                        "{:?}",
                        pilot.operation,
                    );
                }
            }
        }
    }
}

#[test]
fn a_conditional_relation_missing_its_inactive_valid_case_disagrees_with_the_oracle() {
    let pilot = pilot(OperationId::CompactAsh);
    let mut coverage = pilot.coverage.clone();

    for analysis in &mut coverage {
        for operation in analysis.operations.values_mut() {
            operation
                .requirements
                .retain(|key, _| key.case.sponsor != SponsorCase::Absent);
        }
    }

    // The oracle's census names both cases, so the truncated analysis
    // cannot match it.
    for (index, (entry, _, expected)) in pilot.compared().into_iter().enumerate() {
        let _ = entry;
        assert_ne!(coverage[index].project(), expected);
    }

    // Inactive-valid coverage is what proves the sponsorless world is
    // accepted at all: without it no analysis states the disposition.
    assert!(
        !coverage
            .iter()
            .flat_map(PlanCoverageAnalysis::plans)
            .any(|plan| plan.activity == RelationActivity::Vacuous)
    );
}

#[test]
fn a_conditional_relation_missing_its_active_invalid_case_disagrees_with_the_oracle_and_is_rejected()
 {
    let pilot = pilot(OperationId::CompactAsh);
    let mut coverage = pilot.coverage.clone();

    // Every sponsored case dropped: the sponsor relations are then
    // inactive everywhere, and no case states their active rows.
    for analysis in &mut coverage {
        for operation in analysis.operations.values_mut() {
            operation
                .requirements
                .retain(|key, _| key.case.sponsor == SponsorCase::Absent);
        }
    }

    assert!(matches!(
        validate_conditional_coverage(&coverage),
        Err(CompileError::MissingPositiveCoverage { .. }),
    ));

    for (index, (_, _, expected)) in pilot.compared().into_iter().enumerate() {
        assert_ne!(coverage[index].project(), expected);
    }
}

#[test]
fn a_representation_mode_omitted_from_the_plan_set_disagrees_with_the_oracle_and_is_rejected() {
    let pilot = pilot(OperationId::CompactAsh);
    let single = vec![pilot.coverage.first().expect("a plan").clone()];

    assert!(matches!(
        validate_representation_coverage(&pilot.relations, &single),
        Err(CompileError::MissingRepresentationCoverage { .. }),
    ));

    // The oracle expects the complete plan set to fix every allowed
    // mode, one plan at a time: a single plan covers exactly one.
    let mut covered = BTreeSet::new();

    for (entry, _, expected) in pilot.compared() {
        let _ = entry;

        for requirement in expected.requirements.values() {
            covered.extend(
                requirement
                    .positive
                    .iter()
                    .filter_map(|entry| entry.representation),
            );
        }
    }

    let allowed = pilot
        .declarations
        .values()
        .filter_map(|declaration| match &declaration.relation {
            Relation::Representation { allowed, .. } => Some(allowed.clone()),
            _ => None,
        })
        .flatten()
        .collect::<BTreeSet<_>>();

    assert_eq!(covered, allowed);
    assert!(covered.len() > 1);
}

// --- §16.3 determinism ---

#[test]
fn permuted_relation_declarations_agree_with_the_oracle() {
    for pilot in pilots() {
        for (entry, coverage, expected) in pilot.compared() {
            // The relation-case plans carry the declaration order into
            // coverage; reversing them changes nothing the oracle states.
            let mut permuted = entry.clone();
            permuted.relation_case_plans.reverse();

            let again = analyze_placed_coverage(&pilot.relations, &permuted).expect("analysis");

            assert_eq!(again.project(), expected, "{:?}", pilot.operation);
            assert_eq!(again.project(), coverage.project());
        }
    }
}

#[test]
fn permuted_execution_cases_agree_with_the_oracle() {
    for pilot in pilots() {
        for candidate in &pilot.candidates {
            let mut cases =
                crate::case::execution_cases(&pilot.relations, candidate).expect("cases");
            cases.reverse();

            let plans = classify_relation_cases(&pilot.relations, &cases).expect("classification");
            let production = analyze_plan_coverage(&pilot.relations, &plans).expect("coverage");
            let expected = oracle_projection(
                &pilot.declarations,
                &oracle_case_ids(&pilot.declarations, candidate),
                None,
            );

            assert_eq!(production.project(), expected, "{:?}", pilot.operation);
        }
    }
}

#[test]
fn permuted_placements_agree_with_the_oracle() {
    for pilot in pilots() {
        for (entry, _, expected) in pilot.compared() {
            let mut permuted = entry.clone();
            permuted.feasible_placements.reverse();

            let again = analyze_placed_coverage(&pilot.relations, &permuted).expect("analysis");

            assert_eq!(again.project(), expected, "{:?}", pilot.operation);
        }
    }
}

#[test]
fn repeated_analysis_agrees_with_the_oracle_identically() {
    for pilot in pilots() {
        for (entry, coverage, expected) in pilot.compared() {
            let again = analyze_placed_coverage(&pilot.relations, entry).expect("second analysis");

            assert_eq!(&again, coverage, "{:?}", pilot.operation);
            assert_eq!(again.project(), expected, "{:?}", pilot.operation);
        }
    }
}

// --- adversarial synthetic instances and property tests (§12.6) ---

/// One synthetic relation shape.
///
/// Chosen to span the axes coverage branches on: every discharge
/// boundary, both cardinality edges, conditional and unconditional
/// activation, and the hybrid relations that state two boundaries at
/// once. A pilot instance exercises what the pilots happen to declare;
/// these exercise combinations no validated realization assembles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Shape {
    ProtocolCardinality,
    SponsorCardinality,
    ProtocolRecognition,
    SponsorRecognition,
    Conservation,
    OwnerAuthorization,
    Permissionless,
    Constructibility,
    Representation,
    Lifecycle,
    Substrate,
}

const SHAPES: [Shape; 11] = [
    Shape::ProtocolCardinality,
    Shape::SponsorCardinality,
    Shape::ProtocolRecognition,
    Shape::SponsorRecognition,
    Shape::Conservation,
    Shape::OwnerAuthorization,
    Shape::Permissionless,
    Shape::Constructibility,
    Shape::Representation,
    Shape::Lifecycle,
    Shape::Substrate,
];

/// The synthetic operation. One operation throughout: a coverage
/// dependency crossing two of them is rejected outright, and the §12.5
/// cross-operation case covers that on its own.
const SYNTHETIC: OperationId = OperationId::TransferLive;

#[allow(clippy::too_many_lines)]
fn shape_declaration(shape: Shape) -> RelationDeclaration {
    use architecture::AssetId;
    use realization::{Count, ObservedSide};

    let family = |kind, side, object| {
        RelationId::new(
            SYNTHETIC,
            kind,
            RelationSubject::ObjectFamily { side, object },
        )
    };
    let (id, relation) = match shape {
        Shape::ProtocolCardinality => (
            family(
                RelationKind::Cardinality,
                TransactionSide::Input,
                ObjectId::ReceiptLive,
            ),
            Relation::Cardinality {
                side: ObservedSide::Input,
                object: ObjectId::ReceiptLive,
                minimum: Count::ONE,
                maximum: CardinalityMaximum::Exact(Count::new(4)),
            },
        ),

        // Minimum zero and an architecture-owned ceiling: no
        // below-minimum class, and the maximum cited by its bound.
        Shape::SponsorCardinality => (
            family(
                RelationKind::Cardinality,
                TransactionSide::Input,
                ObjectId::PlainLbtc,
            ),
            Relation::Cardinality {
                side: ObservedSide::Input,
                object: ObjectId::PlainLbtc,
                minimum: Count::ZERO,
                maximum: CardinalityMaximum::Bound(architecture::BoundId::FeeSponsorInputMax),
            },
        ),

        Shape::ProtocolRecognition => (
            family(
                RelationKind::Recognition,
                TransactionSide::Input,
                ObjectId::ReceiptLive,
            ),
            Relation::Recognition {
                side: ObservedSide::Input,
                object: ObjectId::ReceiptLive,
                asset: AssetId::U,
            },
        ),

        Shape::SponsorRecognition => (
            family(
                RelationKind::Recognition,
                TransactionSide::Input,
                ObjectId::PlainLbtc,
            ),
            Relation::Recognition {
                side: ObservedSide::Input,
                object: ObjectId::PlainLbtc,
                asset: AssetId::Lbtc,
            },
        ),

        Shape::Conservation => (
            RelationId::new(
                SYNTHETIC,
                RelationKind::Conservation,
                RelationSubject::Asset { asset: AssetId::U },
            ),
            Relation::AmountConservation {
                asset: AssetId::U,
                input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
                output_objects: BTreeSet::from([ObjectId::ReceiptLive]),
            },
        ),

        Shape::OwnerAuthorization => (
            family(
                RelationKind::Authorization,
                TransactionSide::Input,
                ObjectId::ReceiptLive,
            ),
            Relation::OwnerAuthorization {
                object: ObjectId::ReceiptLive,
            },
        ),

        Shape::Permissionless => (
            RelationId::new(
                SYNTHETIC,
                RelationKind::Authorization,
                RelationSubject::Operation,
            ),
            Relation::PermissionlessAuthorization,
        ),

        Shape::Constructibility => (
            RelationId::new(
                SYNTHETIC,
                RelationKind::Constructibility,
                RelationSubject::Operation,
            ),
            Relation::Constructibility {
                class: ConstructibilityClass::PublicPermissionless,
            },
        ),

        Shape::Representation => (
            RelationId::new(
                SYNTHETIC,
                RelationKind::Representation,
                RelationSubject::Representation {
                    object: ObjectId::ReceiptLive,
                },
            ),
            Relation::Representation {
                object: ObjectId::ReceiptLive,
                allowed: BTreeSet::from([
                    RepresentationMode::Explicit,
                    RepresentationMode::PrivateCommitted,
                ]),
            },
        ),

        Shape::Lifecycle => (
            RelationId::new(
                SYNTHETIC,
                RelationKind::Lifecycle,
                RelationSubject::LifecycleExit {
                    object: ObjectId::ReceiptLive,
                    exit: SYNTHETIC,
                },
            ),
            Relation::LifecycleExit {
                object: ObjectId::ReceiptLive,
                exit: SYNTHETIC,
            },
        ),

        Shape::Substrate => (
            RelationId::new(
                SYNTHETIC,
                RelationKind::SubstrateConservation,
                RelationSubject::Asset {
                    asset: AssetId::Lbtc,
                },
            ),
            Relation::SubstrateConservation {
                asset: AssetId::Lbtc,
            },
        ),
    };

    RelationDeclaration {
        id,
        relation,
        proof_alternatives: BTreeSet::new(),
    }
}

/// A synthetic relation analysis over chosen shapes and dependencies.
///
/// Assembled directly rather than derived, so an instance can carry a
/// shape combination — and, for the cycle property, a dependency cycle —
/// that a validated realization never produces.
fn synthetic_relations(
    shapes: &BTreeSet<Shape>,
    edges: &[(usize, usize)],
) -> CompilerRelationAnalysis {
    let declarations = shapes
        .iter()
        .map(|shape| shape_declaration(*shape))
        .collect::<Vec<_>>();
    let mut graph = petgraph::graph::DiGraph::new();
    let mut node_by_id = BTreeMap::new();
    let mut indices = Vec::new();

    for declaration in &declarations {
        let id = AnalysisNodeId::SourceRelation(declaration.id.clone());
        let index = graph.add_node(crate::relation::CompilerRelationNode {
            id: id.clone(),
            source: declaration.clone(),
        });

        node_by_id.insert(id, index);
        indices.push(index);
    }

    for (source, target) in edges {
        graph.add_edge(
            indices[*source],
            indices[*target],
            crate::relation::CompilerRelationEdge::SourceDependency(
                realization::RelationEdge::RecognitionBeforeCardinality,
            ),
        );
    }

    CompilerRelationAnalysis {
        graph,
        node_by_id: node_by_id.clone(),
        evaluation_order: node_by_id.into_keys().collect(),
    }
}

/// One synthetic execution case.
fn synthetic_execution_case(
    sponsor: SponsorCase,
    representation: RepresentationMode,
) -> crate::case::ExecutionCase {
    crate::case::ExecutionCase {
        id: ExecutionCaseId {
            operation: SYNTHETIC,
            sponsor,
            representations: BTreeMap::from([(ObjectId::ReceiptLive, representation)]),
        },
        // No routed source rows: the placement stage is not the subject
        // here, and coverage states its census without them.
        active_sources: Vec::new(),
    }
}

/// The relation-case prerequisite edges and activities of one synthetic
/// coverage analysis, as the closure and component oracles read them.
fn synthetic_dependencies(
    relations: &CompilerRelationAnalysis,
    coverage: &PlanCoverageAnalysis,
) -> (Vec<CoverageNodeId>, Vec<CoverageDependency>) {
    let census = coverage.keys();
    let nodes = census
        .iter()
        .map(|key| CoverageNodeId::RelationCase(key.clone()))
        .collect::<Vec<_>>();
    let edges = oracle_prerequisites(relations, &census)
        .into_iter()
        .map(|(source, target)| CoverageDependency {
            source: CoverageNodeId::RelationCase(source),
            target: CoverageNodeId::RelationCase(target),
            edge: CoverageEdge::RelationPrerequisite,
        })
        .collect::<Vec<_>>();

    (nodes, edges)
}

/// Production and the oracle agree on one synthetic instance.
fn agree_on_synthetic(
    shapes: &BTreeSet<Shape>,
    cases: &[crate::case::ExecutionCase],
    edges: &[(usize, usize)],
) -> CompilerRelationAnalysis {
    let relations = synthetic_relations(shapes, edges);
    let plans = classify_relation_cases(&relations, cases).expect("classification");
    let production = analyze_plan_coverage(&relations, &plans).expect("coverage");
    let declarations = oracle_declarations(&relations);
    let case_ids = cases.iter().map(|case| case.id.clone()).collect();
    let expected = oracle_projection(&declarations, &case_ids, None);

    agree(SYNTHETIC, &production.project(), &expected);
    relations
}

#[test]
fn a_synthetic_instance_of_every_boundary_agrees_with_the_oracle() {
    let shapes = SHAPES.into_iter().collect::<BTreeSet<_>>();
    let cases = vec![
        synthetic_execution_case(SponsorCase::Absent, RepresentationMode::Explicit),
        synthetic_execution_case(SponsorCase::Present, RepresentationMode::Explicit),
        synthetic_execution_case(SponsorCase::Present, RepresentationMode::PrivateCommitted),
    ];

    agree_on_synthetic(&shapes, &cases, &[]);
}

#[test]
fn a_synthetic_cycle_is_rejected_and_the_oracle_finds_the_same_components() {
    let shapes = BTreeSet::from([
        Shape::ProtocolCardinality,
        Shape::ProtocolRecognition,
        Shape::Conservation,
    ]);
    let cases = vec![synthetic_execution_case(
        SponsorCase::Absent,
        RepresentationMode::Explicit,
    )];
    // A three-relation cycle no validated realization admits.
    let relations = synthetic_relations(&shapes, &[(0, 1), (1, 2), (2, 0)]);
    let plans = classify_relation_cases(&relations, &cases).expect("classification");
    let coverage = analyze_plan_coverage(&relations, &plans).expect("coverage");

    let Err(CompileError::CoverageDependencyCycle { components }) =
        analyze_coverage_dependencies(&coverage, &relations)
    else {
        panic!("a coverage cycle is rejected");
    };
    let (nodes, edges) = synthetic_dependencies(&relations, &coverage);

    assert_eq!(components, oracle_components(&nodes, &edges));
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].members.len(), 3);
}

/// One to eight distinct relation shapes.
fn shape_sets() -> impl Strategy<Value = BTreeSet<Shape>> {
    proptest::collection::btree_set(proptest::sample::select(SHAPES.to_vec()), 1..=8)
}

/// One to four distinct execution cases.
fn case_sets() -> impl Strategy<Value = Vec<crate::case::ExecutionCase>> {
    proptest::collection::btree_set(
        proptest::sample::select(vec![
            (SponsorCase::Absent, RepresentationMode::Explicit),
            (SponsorCase::Present, RepresentationMode::Explicit),
            (SponsorCase::Absent, RepresentationMode::PrivateCommitted),
            (SponsorCase::Present, RepresentationMode::PrivateCommitted),
        ]),
        1..=4,
    )
    .prop_map(|selected| {
        selected
            .into_iter()
            .map(|(sponsor, representation)| synthetic_execution_case(sponsor, representation))
            .collect()
    })
}

/// Strictly forward dependency edges over `count` relations, and
/// therefore acyclic by construction.
fn forward_edges(count: usize) -> Vec<(usize, usize)> {
    (0..count)
        .flat_map(|source| ((source + 1)..count).map(move |target| (source, target)))
        .take(4)
        .collect()
}

/// The counterexample the coverage-oracle property search shrank to,
/// written down as its inputs.
///
/// It used to be kept as a generator seed in the regression file beside
/// this suite. A seed replays a case only by asking the same random
/// number generator for it again: it states nothing a reader can read,
/// and it is exactly the kind of value that could be swapped for
/// another without any check noticing. The inputs it shrank to are
/// small and nameable, so they are named here, and the property above
/// keeps searching for new ones.
#[test]
fn the_shrunken_single_conservation_instance_agrees_with_the_oracle() {
    let shapes = BTreeSet::from([Shape::Conservation]);
    let cases = vec![synthetic_execution_case(
        SponsorCase::Absent,
        RepresentationMode::PrivateCommitted,
    )];

    agree_on_synthetic(&shapes, &cases, &[]);

    let edges = forward_edges(shapes.len());
    let relations = agree_on_synthetic(&shapes, &cases, &edges);
    let plans = classify_relation_cases(&relations, &cases).expect("classification");
    let mut coverage = analyze_plan_coverage(&relations, &plans).expect("coverage");

    resolve_coverage_dependencies(&mut coverage, &relations).expect("resolution");

    let census = coverage.keys();
    let pairs = oracle_prerequisites(&relations, &census);
    let activity = oracle_activities(&oracle_declarations(&relations), &census);

    for plan in coverage.plans() {
        let expected = oracle_closure(&pairs, &activity, &plan.key());

        for requirement in &plan.negative {
            if requirement.collateral.policy
                == CollateralPolicy::RequireIntendedAndDependencyClosure
            {
                assert_eq!(requirement.collateral.dependency_closure, expected);
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Bounded generated instances: the production coverage projection
    /// equals the oracle projection, exactly.
    #[test]
    fn generated_instances_agree_with_the_oracle(
        shapes in shape_sets(),
        cases in case_sets(),
    ) {
        agree_on_synthetic(&shapes, &cases, &[]);
    }

    /// Bounded acyclic dependency graphs: coverage resolves, and every
    /// claimed collateral closure equals the repeated-scan fixpoint.
    #[test]
    fn generated_acyclic_dependencies_agree_with_the_closure_oracle(
        shapes in shape_sets(),
        cases in case_sets(),
    ) {
        let edges = forward_edges(shapes.len());
        let relations = agree_on_synthetic(&shapes, &cases, &edges);
        let plans = classify_relation_cases(&relations, &cases).expect("classification");
        let mut coverage = analyze_plan_coverage(&relations, &plans).expect("coverage");

        resolve_coverage_dependencies(&mut coverage, &relations).expect("resolution");

        let census = coverage.keys();
        let pairs = oracle_prerequisites(&relations, &census);
        let activity = oracle_activities(&oracle_declarations(&relations), &census);

        for plan in coverage.plans() {
            let expected = oracle_closure(&pairs, &activity, &plan.key());

            for requirement in &plan.negative {
                if requirement.collateral.policy
                    == CollateralPolicy::RequireIntendedAndDependencyClosure
                {
                    prop_assert_eq!(&requirement.collateral.dependency_closure, &expected);
                }
            }
        }
    }

    /// A generated cycle is rejected by production with canonical
    /// components, and the mutual-reachability oracle finds the same
    /// ones.
    #[test]
    fn generated_cyclic_dependencies_are_rejected_by_both(
        shapes in proptest::collection::btree_set(
            proptest::sample::select(SHAPES.to_vec()),
            2..=6,
        ),
        cases in case_sets(),
    ) {
        let count = shapes.len();
        let mut edges = (0..count - 1)
            .map(|source| (source, source + 1))
            .collect::<Vec<_>>();
        edges.push((count - 1, 0));

        let relations = synthetic_relations(&shapes, &edges);
        let plans = classify_relation_cases(&relations, &cases).expect("classification");
        let coverage = analyze_plan_coverage(&relations, &plans).expect("coverage");
        let (nodes, oracle_edges) = synthetic_dependencies(&relations, &coverage);

        match analyze_coverage_dependencies(&coverage, &relations) {
            Err(CompileError::CoverageDependencyCycle { components }) => {
                prop_assert_eq!(components, oracle_components(&nodes, &oracle_edges));
            }
            other => prop_assert!(false, "a coverage cycle must be rejected: {:?}", other),
        }
    }
}
