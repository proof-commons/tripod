//! Literal announcement censuses and recomputation checks at the plan boundary.

use std::{collections::BTreeSet, sync::LazyLock};

use architecture::{AssetId, ObjectId, OperationId, ProjectionId, ProjectionRule, RootId, RootUse};
use realization::{
    CardinalityMaximum, ConstructibilityClass, Count, ExternalEvidenceRequirement, RelationKind,
    RepresentationMode,
};

use super::{announcement_input, bound_input};
use crate::{
    AnalysisPolicy, AnnouncementMetadataRequirement, BoundCompilerInput, CompileError,
    ConstructorContinuityRequirement, ProofSearchLimits, PublicRecoveryRequirement,
    RootHistoryRequirement,
    analyzed::{ScopedAnalyzedProgram, analyze_scoped_program},
    analyzed_operation::analyze_candidate_operations,
    bind_input,
    case::SponsorCase,
    constructibility::build_constructibility_analysis,
    coverage_graph::{CoverageEdge, CoverageNodeId},
    layout::LayoutRequirement,
    lifecycle::build_lifecycle_analysis,
    maturity_announcement_plan::{
        MaturityAnnouncementRepresentationPlan as Mode, ValidatedMaturityAnnouncementOperationPlan,
        derive_plan, plan_maturity_announcement_target_operation as plan,
        validate_analyzed_maturity_announcement_plan as validate_maturity_announcement_plan,
    },
    placement::{DischargeBoundary, PlacementSearchLimits},
    relation::build_relation_graph,
    requirement::relation_requirements,
    source::RequirementActivation,
};

fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        10_000_000_u64.try_into().unwrap(),
        1_000_000_u64.try_into().unwrap(),
    )
}

static PLAN: LazyLock<ValidatedMaturityAnnouncementOperationPlan> =
    LazyLock::new(|| plan(&announcement_input(), limits()).unwrap());
static ANALYZED: LazyLock<ScopedAnalyzedProgram> =
    LazyLock::new(|| analyze_scoped_program(&announcement_input(), limits()).unwrap());

#[test]
fn derives_the_announcement_with_the_complete_bound_source() {
    let input = announcement_input();
    assert_eq!(PLAN.operation(), OperationId::AnnounceMaturity);
    assert_eq!(PLAN.source().architecture(), input.architecture_binding());
    assert_eq!(*PLAN.source().realization(), input.realization().project());
    assert_eq!(PLAN.source().compilation_scope(), input.scope());
    validate_maturity_announcement_plan(&ANALYZED, &PLAN).unwrap();
}

#[test]
fn independently_built_equal_inputs_produce_equal_typed_plans() {
    let first = announcement_input();
    let second = announcement_input();
    assert_eq!(
        first.realization().project(),
        second.realization().project()
    );
    assert_eq!(first.scope(), second.scope());
    assert_eq!(first.policy(), second.policy());
    assert_eq!(
        plan(&first, limits()).unwrap(),
        plan(&second, limits()).unwrap()
    );
}

#[test]
fn permuted_declarations_and_dependencies_rebuild_the_same_plan() {
    // Bound input has no unchecked declaration builder. Rebuild the assembly's
    // factors through the existing graph and requirement builders, as the
    // analyzed-program permutation tests do; its retained source is canonical.
    let input = announcement_input();
    let mut analyzed = (*ANALYZED).clone();
    let mut nodes = analyzed.source.realization.relations.nodes.clone();
    let mut edges = analyzed.source.realization.relations.edges.clone();
    nodes.reverse();
    edges.reverse();
    let relations = build_relation_graph(input.scope().operations(), &nodes, &edges).unwrap();
    let construction = build_constructibility_analysis(&input).unwrap();
    let lifecycle = build_lifecycle_analysis(&input, &relations).unwrap();
    for candidate in analyzed.proof_plans.values_mut() {
        candidate.relation_requirements = relation_requirements(
            &input,
            &relations,
            &construction,
            &lifecycle,
            &candidate.proof_plan,
        )
        .unwrap();
        candidate.operations = analyze_candidate_operations(
            &relations,
            &candidate.relation_requirements,
            &candidate.proof_plan,
            limits(),
        )
        .unwrap();
    }
    let rebuilt = derive_plan(&analyzed).unwrap();
    validate_maturity_announcement_plan(&analyzed, &rebuilt).unwrap();
    assert_eq!(rebuilt, *PLAN);
}

fn with_policy(policy: AnalysisPolicy) -> BoundCompilerInput {
    let input = announcement_input();
    bind_input(
        &architecture::ARCHITECTURE,
        input.realization().clone(),
        input.scope().clone(),
        policy,
    )
    .unwrap()
}

#[test]
fn different_sufficient_proof_and_placement_limits_preserve_the_plan() {
    let input = with_policy(AnalysisPolicy::strict(ProofSearchLimits::new(
        2_000_000_u64.try_into().unwrap(),
        20_000_u64.try_into().unwrap(),
    )));
    let larger = PlacementSearchLimits::new(
        20_000_000_u64.try_into().unwrap(),
        2_000_000_u64.try_into().unwrap(),
    );
    assert_eq!(plan(&input, larger).unwrap(), *PLAN);
}

#[test]
fn proof_exhaustion_returns_only_the_typed_error() {
    let input = with_policy(AnalysisPolicy::strict(ProofSearchLimits::new(
        1_u64.try_into().unwrap(),
        1_u64.try_into().unwrap(),
    )));
    assert_eq!(
        plan(&input, limits()),
        Err(CompileError::ProofSearchStateLimitExceeded { maximum: 1 })
    );
}

#[test]
fn placement_exhaustion_returns_only_the_typed_error() {
    let tiny = PlacementSearchLimits::new(1_u64.try_into().unwrap(), 1_u64.try_into().unwrap());
    assert_eq!(
        plan(&announcement_input(), tiny),
        Err(CompileError::PlacementSearchStateLimitExceeded { maximum: 1 })
    );
}

#[test]
fn representation_and_relation_case_censuses_are_exact() {
    assert_eq!(
        PLAN.representation().admitted(),
        &BTreeSet::from([Mode::Explicit, Mode::PublicCommitted])
    );
    assert_eq!(
        PLAN.representation().approved(),
        &BTreeSet::from([
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted
        ])
    );
    assert_eq!(Mode::of(RepresentationMode::PrivateCommitted), None);
    assert_eq!(PLAN.representations().count(), 2);
    let mut total = 0;
    for projection in PLAN.representations() {
        assert_eq!(Mode::of(projection.plan().mode()), Some(projection.plan()));
        assert_eq!(projection.relations().count(), 26);
        assert_eq!(projection.cases().count(), 2);
        let count: usize = projection.relations().map(|row| row.cases.len()).sum();
        assert_eq!(count, 52);
        total += count;
        for case in projection.cases() {
            assert_eq!(case.id.operation, OperationId::AnnounceMaturity);
            assert_eq!(
                case.id.representations,
                [(ObjectId::State, projection.plan().mode())].into()
            );
            assert_eq!(
                case.active_relations.len() + case.vacuous_relations.len(),
                26
            );
            assert_eq!(
                case.active_relations.len(),
                if case.id.sponsor == SponsorCase::Absent {
                    22
                } else {
                    26
                }
            );
            assert_eq!(
                projection
                    .relations()
                    .filter(|row| row.cases.contains_key(&case.id))
                    .count(),
                26
            );
        }
    }
    assert_eq!(total, 104);
}

#[test]
fn declaration_relations_and_edges_are_compared_directly() {
    let input = announcement_input();
    let declaration = input
        .realization()
        .operation(OperationId::AnnounceMaturity)
        .unwrap();
    assert_eq!(declaration.relations.len(), 26);
    assert_eq!(declaration.relation_dependencies.len(), 23);
    let ids: BTreeSet<_> = declaration.relations.iter().map(|row| &row.id).collect();
    for projection in PLAN.representations() {
        assert_eq!(
            projection
                .relations()
                .map(|row| &row.relation)
                .collect::<BTreeSet<_>>(),
            ids
        );
    }
    let authored: BTreeSet<_> = declaration
        .relation_dependencies
        .iter()
        .map(|edge| (&edge.prerequisite, &edge.dependent, edge.edge))
        .collect();
    let retained: BTreeSet<_> = PLAN
        .source()
        .realization()
        .relations
        .edges
        .iter()
        .map(|edge| (&edge.source, &edge.target, edge.edge))
        .collect();
    assert_eq!(authored, retained);
    assert_eq!(retained.len(), 23);
}

#[test]
fn generic_source_rows_pin_retention_and_vacuous_erasure() {
    for projection in PLAN.representations() {
        let sources: BTreeSet<_> = projection
            .relations()
            .flat_map(|row| &row.source_requirements)
            .collect();
        assert_eq!(sources.len(), 23);
        assert_eq!(projection.relations().count(), 26);
        // Before relation-case classification the sponsorless case has fifteen
        // active sources; classification erases the vacuous sponsor input bound.
        assert_eq!(
            sources
                .iter()
                .filter(|row| row.activation == RequirementActivation::Always)
                .count(),
            15
        );
        let absent: Vec<_> = projection
            .relations()
            .flat_map(|row| row.cases.values())
            .filter(|row| row.key.case.sponsor == SponsorCase::Absent)
            .collect();
        assert_eq!(
            absent
                .iter()
                .map(|row| row.active_sources.len())
                .sum::<usize>(),
            14
        );
        assert_eq!(
            absent
                .iter()
                .filter(|row| row.boundaries.contains(&DischargeBoundary::RuntimeCarrier))
                .map(|row| row.active_sources.len())
                .sum::<usize>(),
            11
        );
    }
}

#[test]
fn generic_carriers_pin_all_minimal_alternatives() {
    for projection in PLAN.representations() {
        assert_eq!(projection.carriers().count(), 28);
        for (sponsor, expected) in [(SponsorCase::Present, 22), (SponsorCase::Absent, 15)] {
            let alternatives = projection
                .carriers()
                .filter(|row| row.relation_case.case.sponsor == sponsor)
                .map(|row| row.alternatives.len())
                .sum::<usize>();
            assert_eq!(alternatives, expected);
        }
        assert_eq!(
            projection
                .carriers()
                .map(|row| row.alternatives.len())
                .sum::<usize>(),
            37
        );
        assert!(
            projection
                .carriers()
                .flat_map(|row| &row.alternatives)
                .all(|alternative| alternative.carriers.len() == 1)
        );
    }
}

#[test]
fn generic_layout_pins_the_seven_category_counts() {
    for projection in PLAN.representations() {
        let mut counts = [0; 7];
        for row in projection.layout() {
            let category = match row {
                LayoutRequirement::CanonicalCoordinator { .. } => 0,
                LayoutRequirement::AuthenticateFamilyCensus { .. } => 1,
                LayoutRequirement::CompleteAndDisjointFamilies { .. } => 2,
                LayoutRequirement::IsolateSponsorRegion { .. } => 3,
                LayoutRequirement::EnforceRepresentation { .. } => 4,
                LayoutRequirement::MakeSourceAvailable { .. } => 5,
                LayoutRequirement::SecretFreeOperationPath { .. } => 6,
            };
            counts[category] += 1;
        }
        assert_eq!(projection.layout().count(), 59);
        assert_eq!(counts, [3, 8, 8, 4, 1, 35, 0]);
    }
}

#[test]
fn coverage_prerequisites_preserve_each_declared_edge_in_each_case() {
    let declared = &PLAN.source().realization().relations.edges;
    assert_eq!(declared.len(), 23);
    let graph = build_relation_graph(
        &[OperationId::AnnounceMaturity],
        &PLAN.source().realization().relations.nodes,
        declared,
    )
    .unwrap();
    assert_eq!(graph.graph.edge_count(), 23);
    for candidate in ANALYZED.proof_plans.values() {
        let factor = &candidate.operations[&OperationId::AnnounceMaturity];
        let prerequisites: Vec<_> = factor
            .coverage_dependencies
            .edges
            .iter()
            .filter(|edge| edge.edge == CoverageEdge::RelationPrerequisite)
            .collect();
        assert_eq!(prerequisites.len(), 46);
        for case in &factor.execution_cases {
            let actual: BTreeSet<_> = prerequisites
                .iter()
                .filter_map(|edge| match (&edge.source, &edge.target) {
                    (
                        CoverageNodeId::RelationCase(source),
                        CoverageNodeId::RelationCase(target),
                    ) if &source.case == case && &target.case == case => {
                        Some((&source.relation, &target.relation))
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(actual.len(), 23);
            assert_eq!(
                actual,
                declared
                    .iter()
                    .map(|edge| (&edge.source, &edge.target))
                    .collect()
            );
        }
        let mode = Mode::of(
            candidate.proof_plan.representations[&crate::lifecycle::RepresentationChoiceId {
                operation: OperationId::AnnounceMaturity,
                object: ObjectId::State,
            }],
        )
        .unwrap();
        let projection = PLAN.projection(mode).unwrap();
        // The generic boundary retains coverage requirements, while dependency
        // edges stay in the analyzed graph and are checked against the source.
        assert!(
            projection
                .coverage()
                .all(|row| projection.relation(&row.id.relation).is_some())
        );
        assert_eq!(
            projection.coverage().cloned().collect::<Vec<_>>(),
            crate::operation_plan::project_coverage(candidate, factor)
                .unwrap()
                .into_values()
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn retained_requirements_equal_the_owned_announcement_vocabulary() {
    assert_eq!(
        *PLAN.constructibility(),
        ConstructorContinuityRequirement::REQUIRED
    );
    assert_eq!(
        *PLAN.public_facts(),
        AnnouncementMetadataRequirement::public_facts()
    );
    assert_eq!(PLAN.public_facts().len(), 15);
    assert_eq!(
        *PLAN.transition(),
        AnnouncementMetadataRequirement::required()
    );
    assert_eq!(PLAN.transition().fields.len(), 6);
    assert_eq!(*PLAN.root_history(), RootHistoryRequirement::REQUIRED);
    assert_eq!(PLAN.root_history().checks.len(), 9);
    assert_eq!(*PLAN.public_recovery(), PublicRecoveryRequirement::REQUIRED);
    assert_eq!(PLAN.public_recovery().source_facts.len(), 7);
    assert_eq!(PLAN.public_recovery().result_facts.len(), 6);
    assert_eq!(PLAN.public_recovery().inputs.len(), 6);
}

#[test]
fn state_operator_and_policy_projections_retain_the_announcement_contract() {
    assert_eq!(*PLAN.state().object(), ObjectId::State);
    assert_eq!(*PLAN.state().asset(), AssetId::Pid);
    assert_eq!(
        PLAN.state().admitted(),
        &BTreeSet::from([ObjectId::State, ObjectId::PlainLbtc])
    );
    assert_eq!(
        PLAN.state().forbidden(),
        &ObjectId::ALL
            .iter()
            .copied()
            .filter(|object| ![ObjectId::State, ObjectId::PlainLbtc].contains(object))
            .collect()
    );
    for count in [
        PLAN.state().input_cardinality(),
        PLAN.state().output_cardinality(),
    ] {
        assert_eq!(count.minimum(), Count::ONE);
        assert_eq!(count.maximum(), CardinalityMaximum::Exact(Count::ONE));
    }
    assert_eq!(*PLAN.operator().class(), ConstructibilityClass::Operator);
    assert_eq!(
        PLAN.operator().authorization().kind(),
        RelationKind::Authorization
    );
    assert_eq!(
        PLAN.operator().constructibility().kind(),
        RelationKind::Constructibility
    );
    assert_eq!(
        *PLAN.operator().evidence(),
        ExternalEvidenceRequirement::OperatorAuthorization {
            operation: OperationId::AnnounceMaturity
        }
    );
    assert!(PLAN.canonical().expected().is_empty());
    for root in RootId::ALL {
        assert_eq!(
            PLAN.roots().root_use(*root),
            Some(if *root == RootId::State {
                RootUse::Succession
            } else {
                RootUse::Forbidden
            })
        );
    }
    for projection in ProjectionId::ALL {
        assert_eq!(
            PLAN.certificate().rule(*projection),
            Some(if *projection == ProjectionId::TransitionCertificate {
                ProjectionRule::Required
            } else {
                ProjectionRule::Forbidden
            })
        );
    }
    assert_eq!(
        PLAN.lifecycle().implemented().collect::<Vec<_>>(),
        [OperationId::AnnounceMaturity]
    );
    assert_eq!(PLAN.lifecycle().outstanding().count(), 5);
    assert!(!PLAN.lifecycle().release_complete());
    assert!(PLAN.sponsor().is_optional());
    assert_eq!(PLAN.sponsor().envelope_maximum(), Count::ONE);
}

#[test]
fn scope_without_announcement_returns_only_the_typed_scope_error() {
    let input = bound_input(&[OperationId::CompactAsh]);
    assert_eq!(
        plan(&input, limits()),
        Err(CompileError::TargetOperationOutOfScope {
            operation: OperationId::AnnounceMaturity
        })
    );
}

#[test]
fn public_surface_has_no_concrete_target_or_erased_value_fields() {
    let code = include_str!("../maturity_announcement_plan.rs")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    for banned in [
        "digest",
        "Digest",
        "NodeIndex",
        "EdgeIndex",
        "PlanHash",
        "TransactionPosition",
        "transaction_position",
        "output_index",
        "input_index",
        "MetadataBytes",
        "metadata_bytes",
        "metadata_byte_count",
        "METADATA_SIZE",
        "SponsorAmount",
        "sponsor_amount:",
        "sponsor_amount(&self",
        "PrivateKey",
        "Blinder",
        "target_encoding",
        "codec_constant",
    ] {
        assert!(!code.contains(banned), "public plan surface names {banned}");
    }
    for line in code.lines().map(str::trim_start) {
        assert!(!line.starts_with("pub fn new(") && !line.starts_with("pub const fn new("));
        assert!(!line.starts_with("pub fn default("));
    }
    // Public-opening roles belong to public recovery, so opening is deliberately
    // allowed here; the plan carries the requirement vocabulary unchanged.
    assert!(!code.contains("impl Default for ValidatedMaturityAnnouncementOperationPlan"));
}
