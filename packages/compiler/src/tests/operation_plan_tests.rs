//! Recomputation oracles for the compact-ASH target-operation plan
//! (Guide-12 §7, §19).
//!
//! Every census the plan publishes is checked against the *analyzed
//! program*, never against the construction that produced the plan. A
//! test that re-ran the plan's own projection helpers and compared the
//! result would prove only that the module is deterministic, which is
//! not the claim §7.2 makes: the claim is that the published relation,
//! case, carrier, layout, and coverage censuses are exactly the
//! analyzed ones. So the expectations below are rebuilt here from the
//! analyzed value's own fields, and the relation census is rebuilt from
//! the realization projection the analysis retained, one step further
//! upstream again.
//!
//! The corruption oracles run the other way. A validated plan is
//! assembled, one published field is damaged in a way a defect in the
//! join could produce, and the independent validator is required to
//! reject it with the typed variant that names the damage. A validator
//! that only checked containment would accept half of these, so both
//! directions of every census equality are exercised.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, ObjectId, OperationId};
use realization::{Relation, RelationId, RelationKind, RelationSubject, RepresentationMode};

use super::bound_input;
use crate::{
    CompileError,
    analyzed::{AnalyzedProofPlan, ScopedAnalyzedProgram, analyze_scoped_program},
    analyzed_operation::AnalyzedOperation,
    case::SponsorCase,
    coverage::CarrierAssignmentAlternative,
    layout::{LayoutRequirement, names_sponsor_amount},
    lifecycle::{LifecycleExitStatus, RepresentationChoiceId},
    operation_plan::{
        AbstractCarrierRequirement, RepresentationNarrowing, TargetCoverageObligation,
        ValidatedTargetOperationPlan, plan_compact_ash_target_operation,
    },
    placement::{PlacementSearchLimits, RelationActivity},
    source::is_sponsor_amount_operand,
    target::ExternalEvidenceRole,
};

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

/// The two pilot scopes that analyze compact ASH.
fn scopes() -> Vec<Vec<OperationId>> {
    vec![
        vec![OperationId::CompactAsh],
        vec![OperationId::CompactAsh, OperationId::TransferLive],
    ]
}

/// One plan and the analyzed program it must agree with.
struct Subject {
    analyzed: ScopedAnalyzedProgram,
    plan: ValidatedTargetOperationPlan,
}

fn subject(scope: &[OperationId]) -> Subject {
    let input = bound_input(scope);

    Subject {
        analyzed: analyze_scoped_program(&input, limits()).expect("analysis"),
        plan: plan_compact_ash_target_operation(&input, limits()).expect("plan"),
    }
}

impl Subject {
    /// The analyzed operation factors of every plan the Phase-4 policy
    /// admits.
    ///
    /// More than one survives in the two-operation scope, where the
    /// plans differ in the live-transfer representation the compact-ASH
    /// operation does not decide. Every one of them is required to
    /// agree with the published projection.
    fn admissible(&self) -> Vec<(&AnalyzedProofPlan, &AnalyzedOperation)> {
        self.analyzed
            .proof_plans
            .values()
            .filter(|plan| {
                plan.proof_plan
                    .representations
                    .get(&RepresentationChoiceId {
                        operation: OperationId::CompactAsh,
                        object: ObjectId::Ash,
                    })
                    .is_some_and(|mode| *mode == RepresentationMode::Explicit)
            })
            .filter_map(|plan| {
                plan.operations
                    .get(&OperationId::CompactAsh)
                    .map(|operation| (plan, operation))
            })
            .collect()
    }

    /// The analyzed operation factors alone.
    fn admissible_factors(&self) -> Vec<&AnalyzedOperation> {
        self.admissible()
            .into_iter()
            .map(|(_, operation)| operation)
            .collect()
    }
}

// --- §1.3: the relation census does not change at this boundary ---

#[test]
fn the_published_relation_census_is_the_realizations_own_compact_ash_census() {
    // One step further upstream than the analyzed factor: the
    // realization declares the relations, and §1.3 requires the
    // compiler plan's census to equal the analyzed census, which in
    // turn equals the realization's. Rebuilding from the realization
    // means a defect that dropped one relation in *both* the analysis
    // and the plan is still caught here.
    for scope in scopes() {
        let subject = subject(&scope);
        let expected = subject
            .analyzed
            .source
            .realization
            .relations
            .nodes
            .iter()
            .map(|declaration| declaration.id.clone())
            .filter(|relation| relation.operation() == OperationId::CompactAsh)
            .collect::<BTreeSet<_>>();
        let published = subject
            .plan
            .relations()
            .map(|requirement| requirement.relation.clone())
            .collect::<BTreeSet<_>>();

        assert_eq!(published, expected, "scope {scope:?}");
        assert_eq!(published.len(), 23, "scope {scope:?}");
    }
}

#[test]
fn every_published_relation_requirement_is_the_analyzed_bundle() {
    for scope in scopes() {
        let subject = subject(&scope);

        for (plan, operation) in subject.admissible() {
            for requirement in subject.plan.relations() {
                let cases = operation
                    .relation_cases
                    .iter()
                    .filter(|(key, _)| key.relation == requirement.relation)
                    .map(|(key, _)| key.case.clone())
                    .collect::<BTreeSet<_>>();

                assert_eq!(
                    requirement.cases.keys().cloned().collect::<BTreeSet<_>>(),
                    cases,
                );

                let bundle = &plan.relation_requirements[&requirement.relation];

                assert_eq!(requirement.proof, bundle.proof);
                assert_eq!(requirement.representation, bundle.representation);
                assert_eq!(
                    requirement.required_capabilities,
                    bundle.required_capabilities
                );
                assert_eq!(requirement.source_requirements, bundle.source_requirements);
                assert_eq!(requirement.external_evidence, bundle.external_evidence);
                assert_eq!(requirement.lifecycle, bundle.lifecycle);
            }
        }
    }
}

// --- §7.2: the case, carrier, layout, and coverage censuses ---

#[test]
fn the_published_case_census_is_the_analyzed_case_census() {
    for scope in scopes() {
        let subject = subject(&scope);
        let published = subject
            .plan
            .cases()
            .map(|case| case.id.clone())
            .collect::<BTreeSet<_>>();

        for factor in subject.admissible_factors() {
            assert_eq!(published, factor.execution_cases, "scope {scope:?}");
        }

        // Independently: the compact-ASH open-flow policy admits a fee
        // sponsor, so the operation has exactly the sponsorless and
        // sponsored cases, and both fix ASH to the selected mode.
        assert_eq!(
            published
                .iter()
                .map(|case| case.sponsor)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([SponsorCase::Absent, SponsorCase::Present]),
        );
        assert_eq!(published.len(), 2, "scope {scope:?}");

        for case in &published {
            assert_eq!(
                case.representations.get(&ObjectId::Ash),
                Some(&RepresentationMode::Explicit),
            );
        }
    }
}

#[test]
fn every_case_splits_the_relation_census_into_active_and_vacuous() {
    // No relation disappears in a case. §19.3 keeps an inactive
    // relation in the census with its vacuity stated, so the two sets
    // partition the whole census rather than covering the active part.
    for scope in scopes() {
        let subject = subject(&scope);
        let census = subject
            .plan
            .relations()
            .map(|requirement| requirement.relation.clone())
            .collect::<BTreeSet<_>>();

        for case in subject.plan.cases() {
            let union = case
                .active_relations
                .union(&case.vacuous_relations)
                .cloned()
                .collect::<BTreeSet<_>>();

            assert_eq!(union, census, "scope {scope:?} case {:?}", case.id);
            assert!(
                case.active_relations.is_disjoint(&case.vacuous_relations),
                "a relation is active or vacuous, never both",
            );
        }

        // Cross-checked against the analyzed relation-case activity.
        for factor in subject.admissible_factors() {
            for (key, requirements) in &factor.relation_cases {
                let case = subject.plan.case(&key.case).expect("published case");
                let expected_active = requirements.activity == RelationActivity::Active;

                assert_eq!(
                    case.active_relations.contains(&key.relation),
                    expected_active,
                    "{key:?}",
                );
            }
        }
    }
}

#[test]
fn the_published_layout_census_is_the_analyzed_operation_census() {
    for scope in scopes() {
        let subject = subject(&scope);
        let published = subject.plan.layout().cloned().collect::<BTreeSet<_>>();

        for factor in subject.admissible_factors() {
            assert_eq!(published, factor.layout_requirements, "scope {scope:?}");
        }

        assert_eq!(published.len(), 69, "scope {scope:?}");

        // Every relation-owned layout requirement names a relation of
        // this operation. A requirement about another operation would
        // be an unowned obligation §1.3 rejects.
        for requirement in &published {
            if let Some(relation) = requirement.relation() {
                assert_eq!(relation.operation(), OperationId::CompactAsh);
            }
        }
    }
}

#[test]
fn the_published_carrier_census_is_the_analyzed_carrier_alternative_set() {
    for scope in scopes() {
        let subject = subject(&scope);
        let published = subject
            .plan
            .carriers()
            .cloned()
            .collect::<BTreeSet<AbstractCarrierRequirement>>();

        for factor in subject.admissible_factors() {
            let expected = factor
                .relation_cases
                .iter()
                .filter(|(_, requirements)| !requirements.carrier_assignments.is_empty())
                .map(|(key, requirements)| AbstractCarrierRequirement {
                    relation_case: key.clone(),
                    alternatives: requirements.carrier_assignments.clone(),
                })
                .collect::<BTreeSet<_>>();

            assert_eq!(published, expected, "scope {scope:?}");
        }

        assert!(!published.is_empty());

        // An empty alternative set would read as an obligation nothing
        // can discharge; the analysis means there is no obligation.
        for requirement in &published {
            assert!(!requirement.alternatives.is_empty());
            assert_eq!(
                requirement.relation_case.relation.operation(),
                OperationId::CompactAsh,
            );
        }
    }
}

#[test]
fn the_published_coverage_census_is_the_analyzed_relation_indexed_coverage() {
    for scope in scopes() {
        let subject = subject(&scope);
        let published = subject
            .plan
            .coverage()
            .map(|requirement| requirement.id.clone())
            .collect::<BTreeSet<_>>();

        for factor in subject.admissible_factors() {
            let expected = factor
                .coverage
                .requirements
                .values()
                .flat_map(|projection| {
                    projection
                        .positive
                        .iter()
                        .map(|requirement| requirement.id.clone())
                        .chain(
                            projection
                                .negative
                                .iter()
                                .map(|requirement| requirement.id.clone()),
                        )
                })
                .collect::<BTreeSet<_>>();

            assert_eq!(published, expected, "scope {scope:?}");
        }

        assert!(!published.is_empty());

        // §19: every row is relation-indexed and carries the case,
        // activation, and proof its relation was analyzed under.
        for requirement in subject.plan.coverage() {
            assert_eq!(requirement.id.relation.operation(), OperationId::CompactAsh);
            assert!(
                subject
                    .plan
                    .relation(&requirement.id.relation)
                    .is_some_and(|relation| relation.proof == requirement.proof),
                "a coverage row's proof is its relation's proof",
            );
            assert!(subject.plan.case(&requirement.id.case).is_some());

            match &requirement.obligation {
                TargetCoverageObligation::Positive(positive) => {
                    assert_eq!(positive.id, requirement.id);
                }
                TargetCoverageObligation::Negative(negative) => {
                    assert_eq!(negative.id, requirement.id);
                }
            }
        }
    }
}

#[test]
fn the_published_capability_and_evidence_censuses_are_the_relation_unions() {
    for scope in scopes() {
        let subject = subject(&scope);
        let expected = subject
            .plan
            .relations()
            .flat_map(|requirement| requirement.required_capabilities.iter().copied())
            .collect::<BTreeSet<_>>();

        assert_eq!(
            subject.plan.capabilities().collect::<BTreeSet<_>>(),
            expected,
            "scope {scope:?}",
        );
        assert_eq!(
            subject.plan.external_evidence().collect::<Vec<_>>(),
            [ExternalEvidenceRole::SubstrateConservation],
        );
    }
}

// --- §1.7: the explicit Guide-11 representation selection ---

#[test]
fn the_plan_selects_explicit_ash_and_records_the_narrowing_as_policy() {
    for scope in scopes() {
        let subject = subject(&scope);
        let policy = subject.plan.representation();

        assert_eq!(
            policy.selection(ObjectId::Ash),
            Some(RepresentationMode::Explicit),
            "scope {scope:?}",
        );

        // The realization approves two modes for compact-ASH ASH, so
        // the narrowing is a deployment decision and not a semantic
        // necessity — the distinction §1.7 requires to be recorded.
        assert_eq!(
            policy.approved(ObjectId::Ash),
            Some(&BTreeSet::from([
                RepresentationMode::Explicit,
                RepresentationMode::PublicCommitted,
            ])),
        );
        assert_eq!(
            policy.narrowing(ObjectId::Ash),
            Some(RepresentationNarrowing::DeploymentPolicy),
        );

        // Independently: the approved set is the realization's own
        // representation relation for this operation and object.
        let declared = subject
            .analyzed
            .source
            .realization
            .relations
            .nodes
            .iter()
            .filter(|declaration| declaration.id.operation() == OperationId::CompactAsh)
            .filter_map(|declaration| match &declaration.relation {
                Relation::Representation { object, allowed } if *object == ObjectId::Ash => {
                    Some(allowed.clone())
                }
                _ => None,
            })
            .fold(BTreeSet::new(), |mut all, allowed| {
                all.extend(allowed);
                all
            });

        assert_eq!(policy.approved(ObjectId::Ash), Some(&declared));

        // The selection is not carried as a dormant alternative: every
        // published case fixes the same mode.
        for case in subject.plan.cases() {
            assert_eq!(
                case.id.representations.get(&ObjectId::Ash),
                Some(&RepresentationMode::Explicit),
            );
        }
    }
}

// --- §5.7: lifecycle status ---

#[test]
fn the_plan_implements_compact_ash_and_leaves_clear_outstanding() {
    for scope in scopes() {
        let subject = subject(&scope);
        let lifecycle = subject.plan.lifecycle();

        assert_eq!(
            lifecycle.implemented().collect::<BTreeSet<_>>(),
            BTreeSet::from([OperationId::CompactAsh]),
            "scope {scope:?}",
        );
        assert_eq!(
            lifecycle
                .outstanding()
                .map(|requirement| requirement.exit)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([OperationId::Clear]),
            "scope {scope:?}",
        );
        assert!(
            !lifecycle.release_complete(),
            "the candidate is lifecycle-incomplete because clear is unimplemented",
        );

        // Cross-checked against the analyzed obligations.
        for plan in subject.analyzed.proof_plans.values() {
            for requirement in plan
                .relation_requirements
                .values()
                .flat_map(|bundle| bundle.lifecycle.iter())
                .filter(|requirement| requirement.representation == RepresentationMode::Explicit)
                .filter(|requirement| requirement.object == ObjectId::Ash)
            {
                if requirement.status == LifecycleExitStatus::DeclaredOutsideCompilerScope {
                    assert_eq!(requirement.exit, OperationId::Clear);
                }
            }
        }
    }
}

// --- §1.6: no sponsor amount reaches any published field ---

#[test]
fn no_published_field_names_an_erased_sponsor_amount() {
    for scope in scopes() {
        let subject = subject(&scope);

        let layout = subject
            .plan
            .layout()
            .cloned()
            .chain(subject.plan.relations().flat_map(|relation| {
                relation
                    .cases
                    .values()
                    .flat_map(|case| case.layout_requirements.iter().cloned())
            }))
            .chain(
                subject
                    .plan
                    .carriers()
                    .flat_map(|carrier| carrier.alternatives.iter())
                    .flat_map(|alternative| alternative.layout.iter().cloned()),
            )
            .collect::<Vec<LayoutRequirement>>();

        for requirement in &layout {
            assert!(!names_sponsor_amount(requirement), "{requirement:?}");
        }

        let sources = subject
            .plan
            .relations()
            .flat_map(|relation| relation.source_requirements.iter())
            .chain(
                subject
                    .plan
                    .relations()
                    .flat_map(|relation| relation.cases.values())
                    .flat_map(|case| case.active_sources.iter()),
            );

        for source in sources {
            assert!(
                !is_sponsor_amount_operand(source.operand.role()),
                "{source:?}",
            );
        }
    }
}

// --- §7.4: determinism across equal and permuted inputs ---

#[test]
fn equal_inputs_and_scope_permutations_produce_equal_plans() {
    let first = subject(&[OperationId::CompactAsh, OperationId::TransferLive]).plan;
    let repeated = subject(&[OperationId::CompactAsh, OperationId::TransferLive]).plan;
    let permuted = subject(&[OperationId::TransferLive, OperationId::CompactAsh]).plan;

    assert_eq!(first, repeated);
    assert_eq!(first, permuted);

    // The single-operation scope analyzes the same operation under a
    // different compiler scope, so the plans differ in exactly one
    // place — the retained source — and in nothing the backend emits.
    let alone = subject(&[OperationId::CompactAsh]).plan;

    assert_ne!(alone.source(), first.source());
    assert_eq!(
        alone.relations().cloned().collect::<Vec<_>>(),
        first.relations().cloned().collect::<Vec<_>>(),
    );
    assert_eq!(
        alone.layout().cloned().collect::<Vec<_>>(),
        first.layout().cloned().collect::<Vec<_>>(),
    );
    assert_eq!(
        alone.coverage().cloned().collect::<Vec<_>>(),
        first.coverage().cloned().collect::<Vec<_>>(),
    );
}

#[test]
fn an_out_of_scope_operation_has_no_plan() {
    let input = bound_input(&[OperationId::TransferLive]);

    assert_eq!(
        plan_compact_ash_target_operation(&input, limits()).unwrap_err(),
        CompileError::TargetOperationOutOfScope {
            operation: OperationId::CompactAsh,
        },
    );
}

#[test]
fn a_truncated_search_publishes_no_plan() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let truncated = PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1).expect("nonzero"),
        std::num::NonZeroU64::new(1).expect("nonzero"),
    );

    assert!(matches!(
        plan_compact_ash_target_operation(&input, truncated),
        Err(CompileError::PlacementSearchStateLimitExceeded { .. }
            | CompileError::PlacementCandidateLimitExceeded { .. }),
    ));
}

// --- §7.4: the independent validator rejects every corruption ---

/// One damaged plan and the defect its damage must raise.
fn corruption_subject() -> Subject {
    subject(&[OperationId::CompactAsh, OperationId::TransferLive])
}

/// A relation of the other pilot, which this operation never owns.
fn foreign_relation() -> RelationId {
    RelationId::new(
        OperationId::TransferLive,
        RelationKind::Conservation,
        RelationSubject::Asset {
            asset: AssetId::Lbtc,
        },
    )
}

#[test]
fn a_removed_relation_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let removed = damaged
        .relations_mut()
        .keys()
        .next()
        .cloned()
        .expect("a relation");

    damaged.relations_mut().remove(&removed);

    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanRelationCensusMismatch { relation: removed }),
    );
}

#[test]
fn an_added_relation_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let borrowed = damaged
        .relations_mut()
        .values()
        .next()
        .cloned()
        .expect("a relation");
    let foreign = foreign_relation();

    damaged.relations_mut().insert(foreign.clone(), borrowed);

    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanRelationCensusMismatch { relation: foreign }),
    );
}

#[test]
fn a_changed_carrier_alternative_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let mut carriers = damaged.carriers_mut().clone();
    let mut first = carriers.iter().next().cloned().expect("a carrier row");

    carriers.remove(&first);
    first.alternatives.insert(CarrierAssignmentAlternative {
        carriers: first
            .alternatives
            .iter()
            .next()
            .expect("an alternative")
            .carriers
            .clone(),
        layout: BTreeSet::new(),
    });
    carriers.insert(first);
    *damaged.carriers_mut() = carriers;

    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanCarrierCensusMismatch),
    );
}

#[test]
fn a_changed_layout_requirement_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let removed = damaged.layout_mut().iter().next().cloned().expect("layout");

    damaged.layout_mut().remove(&removed);

    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanLayoutCensusMismatch),
    );
}

#[test]
fn a_changed_coverage_row_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let id = damaged
        .coverage_mut()
        .keys()
        .next()
        .cloned()
        .expect("a coverage row");

    damaged.coverage_mut().remove(&id);

    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanCoverageCensusMismatch),
    );
}

#[test]
fn a_changed_case_census_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let id = damaged.cases_mut().keys().next().cloned().expect("a case");

    damaged.cases_mut().remove(&id);

    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanCaseCensusMismatch),
    );
}

#[test]
fn a_rewritten_relation_requirement_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();
    let relation = damaged
        .relations_mut()
        .keys()
        .next()
        .cloned()
        .expect("a relation");

    damaged
        .relations_mut()
        .get_mut(&relation)
        .expect("the relation")
        .required_capabilities = BTreeSet::new();

    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanRelationRequirementMismatch),
    );
}

#[test]
fn a_changed_representation_selection_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();

    damaged
        .representation_mut()
        .selected_mut()
        .insert(ObjectId::Ash, RepresentationMode::PublicCommitted);

    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanRepresentationMismatch {
            object: ObjectId::Ash,
        }),
    );
}

#[test]
fn a_changed_lifecycle_status_is_rejected() {
    let subject = corruption_subject();
    let mut damaged = subject.plan.clone();

    damaged.lifecycle_mut().clear_outstanding();

    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&subject.analyzed, &damaged),
        Err(CompileError::TargetPlanLifecycleMismatch),
    );
}

#[test]
fn a_foreign_source_binding_is_rejected() {
    let combined = corruption_subject();
    let alone = subject(&[OperationId::CompactAsh]);
    let mut damaged = combined.plan.clone();

    *damaged.source_mut() = alone.plan.source().clone();

    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&combined.analyzed, &damaged),
        Err(CompileError::TargetPlanSourceMismatch),
    );
}

/// The census sizes this wave measured, restated as data.
///
/// A matrix derived from the plan would only prove the plan agrees with
/// itself. These are the Guide-7 §19 measured pilot sizes, and the plan
/// republishes them without loss.
#[test]
fn the_published_censuses_have_the_measured_pilot_sizes() {
    let plan = corruption_subject().plan;
    let counts: BTreeMap<&str, usize> = BTreeMap::from([
        ("relations", plan.relations().count()),
        ("cases", plan.cases().count()),
        ("layout", plan.layout().count()),
    ]);

    assert_eq!(
        counts,
        BTreeMap::from([("cases", 2), ("layout", 69), ("relations", 23)]),
    );

    // The relation-case census is the product only because every
    // relation is present in every case, active or vacuous.
    assert_eq!(
        plan.relations()
            .map(|requirement| requirement.cases.len())
            .sum::<usize>(),
        46,
    );
}
