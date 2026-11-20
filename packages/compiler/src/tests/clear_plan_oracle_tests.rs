//! Clear's local placement oracle. Literal relation keys and carrier roles
//! come from the declared Clear contract; source routing is decided here from
//! typed source classes, without calling carrier eligibility or reading the
//! product to construct an expectation. No complete Clear set is expanded.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use realization::{
    AvailabilityClass, RelationId, RelationKind, RelationSubject, RepresentationMode, StateField,
    TransactionSide,
};

use crate::{
    analyzed::analyze_scoped_program,
    analyzed_operation::{PlacementFactorOption, PlacementProduct},
    carrier::{
        CarrierAvailability, CarrierQuantification, CarrierRole, EligibleCarrier, SourceRouting,
    },
    case::{ExecutionCaseId, SponsorCase},
    clear_operation_plan::{derive_clear_plan, validate_clear_plan},
    layout::LayoutRequirement,
    placement::{
        PlacedCarrier, PlacementAssignment, PlacementCandidate, PlacementSearchLimits,
        RelationCaseKey,
    },
    source::{OperandRole, RequiredSourceKind, SourceRequirement},
};

const CLEAR: OperationId = OperationId::Clear;

fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        10_000_000_u64.try_into().unwrap(),
        1_000_000_u64.try_into().unwrap(),
    )
}

fn case(
    sponsor: SponsorCase,
    state: RepresentationMode,
    ash: RepresentationMode,
) -> ExecutionCaseId {
    ExecutionCaseId {
        operation: CLEAR,
        sponsor,
        representations: BTreeMap::from([(ObjectId::State, state), (ObjectId::Ash, ash)]),
        side_representations: Box::new(BTreeMap::new()),
    }
}

const fn placed(carrier: CarrierRole, quantification: CarrierQuantification) -> PlacedCarrier {
    PlacedCarrier {
        carrier,
        quantification,
    }
}

fn globals(quantification: CarrierQuantification) -> [PlacedCarrier; 2] {
    [ObjectId::State, ObjectId::Ash].map(|anchor| {
        placed(
            CarrierRole::OperationGlobal {
                operation: CLEAR,
                anchor,
            },
            quantification,
        )
    })
}

fn insert_factor(
    factors: &mut BTreeMap<RelationCaseKey, BTreeSet<PlacedCarrier>>,
    case: &ExecutionCaseId,
    kind: RelationKind,
    subject: RelationSubject,
    choices: impl IntoIterator<Item = PlacedCarrier>,
) {
    let key = RelationCaseKey {
        relation: RelationId::new(CLEAR, kind, subject),
        case: case.clone(),
    };
    assert!(factors.insert(key, choices.into_iter().collect()).is_none());
}

fn literal_carriers(
    state: RepresentationMode,
    ash: RepresentationMode,
) -> BTreeMap<RelationCaseKey, BTreeSet<PlacedCarrier>> {
    use CarrierQuantification::{CompleteFamilyProof, PerMember, Single};
    use RelationKind as K;
    use RelationSubject as S;
    let mut table = BTreeMap::new();
    for sponsor in [SponsorCase::Absent, SponsorCase::Present] {
        let case = case(sponsor, state, ash);
        for side in [TransactionSide::Input, TransactionSide::Output] {
            for object in [ObjectId::State, ObjectId::Ash, ObjectId::PlainLbtc] {
                if object == ObjectId::PlainLbtc && sponsor == SponsorCase::Absent {
                    continue;
                }
                let mut cardinality = globals(Single).to_vec();
                let mut recognition = globals(if side == TransactionSide::Input {
                    CompleteFamilyProof
                } else {
                    Single
                })
                .to_vec();
                if side == TransactionSide::Input {
                    cardinality.push(placed(
                        CarrierRole::InputFamilyCoordinator { object },
                        Single,
                    ));
                    recognition.push(placed(
                        CarrierRole::InputFamilyCoordinator { object },
                        CompleteFamilyProof,
                    ));
                    recognition.push(placed(
                        CarrierRole::EveryInputFamilyMember { object },
                        PerMember,
                    ));
                }
                let subject = S::ObjectFamily { side, object };
                insert_factor(
                    &mut table,
                    &case,
                    K::Cardinality,
                    subject.clone(),
                    cardinality,
                );
                insert_factor(&mut table, &case, K::Recognition, subject, recognition);
            }
            insert_factor(
                &mut table,
                &case,
                K::AllowedObjectFamilies,
                S::TransactionSide { side },
                globals(Single),
            );
        }
        for kind in [
            K::CanonicalDeltaPolicy,
            K::OpenFlowPolicy,
            K::SponsorEnvelopeMultiplicity,
            K::SponsorIsolation,
            K::RootPolicy,
            K::ProjectionPolicy,
            K::ClearClamp,
            K::ClearFloor,
            K::ClearFlow,
            K::ClearBinding,
        ] {
            let subject = if matches!(kind, K::SponsorEnvelopeMultiplicity | K::SponsorIsolation) {
                S::Sponsor
            } else {
                S::Operation
            };
            insert_factor(&mut table, &case, kind, subject, globals(Single));
        }
        for field in StateField::ALL {
            insert_factor(
                &mut table,
                &case,
                K::ClearAssignment,
                S::StateField { field: *field },
                globals(Single),
            );
        }
    }
    assert_eq!(table.len(), 56);
    assert_eq!(table.values().map(BTreeSet::len).sum::<usize>(), 127);
    table
}

// Decide routing from the typed source class and the local role. This is
// deliberately separate from carrier::relation_case_eligibility, which the
// production lowering calls. All Clear runtime sources are public or confined
// sponsor evidence; a new witness class requires an explicit oracle update.
fn routing(
    source: &SourceRequirement,
    carrier: &CarrierRole,
    sponsor: SponsorCase,
) -> CarrierAvailability {
    match source.availability {
        AvailabilityClass::Public => match source.source {
            RequiredSourceKind::RuntimeArchitectureBound
            | RequiredSourceKind::PublicConstructionData
            | RequiredSourceKind::DerivedExpression => CarrierAvailability::Intrinsic,
            RequiredSourceKind::AuthenticatedInputObject => {
                if matches!(
                    (carrier, source.operand.role()),
                    (
                        CarrierRole::EveryInputFamilyMember { object: family },
                        OperandRole::ObjectFamilyMembers {
                            side: TransactionSide::Input,
                            object,
                        },
                    ) if family == object
                ) {
                    CarrierAvailability::Intrinsic
                } else {
                    CarrierAvailability::LayoutProvided
                }
            }
            RequiredSourceKind::AuthenticatedOutputObject
            | RequiredSourceKind::AuthenticatedFamilyCensus
            | RequiredSourceKind::AuthenticatedConsensusValue
            | RequiredSourceKind::AuthenticatedCommitmentRelation
            | RequiredSourceKind::AuthenticatedTransitionCertificate => {
                CarrierAvailability::LayoutProvided
            }
            RequiredSourceKind::InputOwnerWitness
            | RequiredSourceKind::OperatorWitness
            | RequiredSourceKind::RefundKeyWitness
            | RequiredSourceKind::SponsorLocalWitness
            | RequiredSourceKind::ExternalEvidence => {
                panic!("Clear runtime relation has unavailable public witness")
            }
        },
        AvailabilityClass::SponsorLocal => {
            assert_eq!(sponsor, SponsorCase::Present);
            if matches!(
                carrier,
                CarrierRole::InputFamilyCoordinator {
                    object: ObjectId::PlainLbtc,
                } | CarrierRole::EveryInputFamilyMember {
                    object: ObjectId::PlainLbtc,
                }
            ) {
                CarrierAvailability::Intrinsic
            } else {
                CarrierAvailability::SponsorRegionConfined
            }
        }
        AvailabilityClass::InputOwners { .. }
        | AvailabilityClass::Operator
        | AvailabilityClass::RefundKey { .. }
        | AvailabilityClass::ClientOwners { .. } => {
            panic!("Clear runtime relation has an unexpected private source")
        }
    }
}

fn expected_layout(
    key: &RelationCaseKey,
    choice: &PlacedCarrier,
    routings: &[SourceRouting],
) -> BTreeSet<LayoutRequirement> {
    use LayoutRequirement as L;
    let mut layout = BTreeSet::new();
    if matches!(
        choice.carrier,
        CarrierRole::OperationGlobal { .. } | CarrierRole::InputFamilyCoordinator { .. }
    ) {
        layout.insert(L::CanonicalCoordinator {
            operation: CLEAR,
            anchor: choice.carrier.clone(),
        });
    }
    if choice.quantification == CarrierQuantification::CompleteFamilyProof
        && let RelationSubject::ObjectFamily { side, object } = key.relation.subject()
    {
        layout.insert(L::AuthenticateFamilyCensus {
            relation: key.relation.clone(),
            side: *side,
            object: *object,
        });
        layout.insert(L::CompleteAndDisjointFamilies {
            relation: key.relation.clone(),
            side: *side,
        });
    }
    for route in routings {
        match route.availability {
            CarrierAvailability::Intrinsic => {}
            CarrierAvailability::LayoutProvided => {
                layout.insert(L::MakeSourceAvailable {
                    relation: key.relation.clone(),
                    case: key.case.clone(),
                    carrier: choice.carrier.clone(),
                    source: route.source.clone(),
                });
            }
            CarrierAvailability::SponsorRegionConfined => {
                layout.insert(L::IsolateSponsorRegion {
                    relation: key.relation.clone(),
                    case: key.case.clone(),
                });
            }
        }
    }
    layout
}

fn expected_product(
    plans: &[crate::placement::RelationCasePlan],
    state: RepresentationMode,
    ash: RepresentationMode,
) -> (
    PlacementProduct,
    BTreeMap<RelationCaseKey, Vec<EligibleCarrier>>,
) {
    let mut expected = BTreeMap::new();
    let mut routes = BTreeMap::new();
    for (key, choices) in literal_carriers(state, ash) {
        let plan = plans
            .iter()
            .find(|plan| plan.relation == key.relation && plan.case == key.case)
            .unwrap_or_else(|| {
                panic!(
                    "literal Clear relation case {key:?}; cases {:?}",
                    plans.iter().map(|row| &row.case).collect::<BTreeSet<_>>()
                )
            });
        let requirement = plan
            .runtime_requirements
            .first()
            .expect("literal runtime case");
        assert_eq!(plan.runtime_requirements.len(), 1);
        let mut eligible = Vec::new();
        let options = choices
            .into_iter()
            .map(|choice| {
                let mut routings = requirement
                    .sources
                    .iter()
                    .map(|source| SourceRouting {
                        source: source.clone(),
                        availability: routing(source, &choice.carrier, key.case.sponsor),
                    })
                    .collect::<Vec<_>>();
                routings.sort();
                let layout = expected_layout(&key, &choice, &routings);
                eligible.push(EligibleCarrier {
                    carrier: choice.carrier.clone(),
                    quantification: choice.quantification,
                    routings,
                });
                PlacementFactorOption {
                    carriers: BTreeSet::from([choice]),
                    layout_requirements: layout,
                }
            })
            .collect::<BTreeSet<_>>();
        eligible.sort();
        assert!(routes.insert(key.clone(), eligible).is_none());
        assert!(expected.insert(key, options).is_none());
    }
    (PlacementProduct { factors: expected }, routes)
}

fn selected_member(product: &PlacementProduct, selection: usize) -> PlacementCandidate {
    let mut assignments = Vec::new();
    let mut layout = BTreeSet::new();
    for (index, (key, options)) in product.factors.iter().enumerate() {
        let option = match selection {
            0 => options.first().unwrap(),
            1 => options.last().unwrap(),
            _ if index.is_multiple_of(2) => options.first().unwrap(),
            _ => options.last().unwrap(),
        };
        assignments.push(PlacementAssignment {
            relation: key.relation.clone(),
            case: key.case.clone(),
            carriers: option.carriers.iter().cloned().collect(),
        });
        layout.extend(option.layout_requirements.iter().cloned());
    }
    PlacementCandidate {
        assignments,
        layout_requirements: layout.into_iter().collect(),
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Literal source, factor, routing and member claims form one oracle.
fn clear_plan_matches_small_oracle() {
    let control = super::source_tests::clear_control();
    let analyzed = analyze_scoped_program(&control.input, limits()).expect("complete Clear");
    let facade = derive_clear_plan(&analyzed).expect("Clear facade");
    assert_eq!(validate_clear_plan(&analyzed, &facade), Ok(()));
    assert_eq!(facade.relations().count(), 44);
    assert_eq!(
        facade
            .cases()
            .map(|row| row.id.clone())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            case(
                SponsorCase::Absent,
                RepresentationMode::Explicit,
                RepresentationMode::Explicit
            ),
            case(
                SponsorCase::Present,
                RepresentationMode::Explicit,
                RepresentationMode::Explicit
            )
        ])
    );
    let declared = control
        .input
        .realization()
        .project()
        .relations
        .nodes
        .into_iter()
        .filter(|row| row.id.operation() == CLEAR)
        .map(|row| row.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(declared.len(), 44);
    assert_eq!(
        facade
            .relations()
            .map(|row| row.relation.clone())
            .collect::<BTreeSet<_>>(),
        declared
    );

    let (records, stages) = crate::engine_attestation::program::test_preparation(&control.input)
        .expect("Clear preparation");
    let clear = crate::engine_attestation::operation_key(CLEAR);
    let engine = crate::engine_program::analyze_prepared_scoped(
        &records,
        control.input.policy().proof_search_limits,
        limits(),
        Some(stages),
        &BTreeSet::from([clear]),
    )
    .expect("complete Clear engine");
    assert_eq!(analyzed.proof_plans.len(), 4);
    for proof in analyzed.proof_plans.values() {
        let selected = &proof.proof_plan.representations;
        let state = selected[&crate::lifecycle::RepresentationChoiceId {
            operation: CLEAR,
            object: ObjectId::State,
            side: None,
        }];
        let ash = selected[&crate::lifecycle::RepresentationChoiceId {
            operation: CLEAR,
            object: ObjectId::Ash,
            side: None,
        }];
        let factor = &proof.operations[&CLEAR];
        let product = factor.feasible_placements.product().expect("Clear product");
        let cases = crate::case::execution_cases(&control.relations, &proof.proof_plan)
            .expect("Clear cases");
        let plans = crate::placement::classify_relation_cases(&control.relations, &cases)
            .expect("Clear discharge plans");
        let (expected, expected_routes) = expected_product(&plans, state, ash);
        assert_eq!(product, &expected);
        if state == RepresentationMode::Explicit && ash == RepresentationMode::Explicit {
            let expected_carriers = expected
                .factors
                .iter()
                .map(|(key, options)| {
                    (
                        key.clone(),
                        options
                            .iter()
                            .map(|option| crate::coverage::CarrierAssignmentAlternative {
                                carriers: option.carriers.clone(),
                                layout: option.layout_requirements.clone(),
                            })
                            .collect::<BTreeSet<_>>(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let facade_carriers = facade
                .carriers()
                .map(|row| (row.relation_case.clone(), row.alternatives.clone()))
                .collect::<BTreeMap<_, _>>();
            assert_eq!(facade_carriers, expected_carriers);
        }
        let eligibility = crate::carrier::relation_case_eligibility(&control.relations, &plans)
            .expect("production routing subject");
        let actual_routes = eligibility
            .into_iter()
            .map(|row| {
                (
                    RelationCaseKey {
                        relation: row.relation,
                        case: row.case,
                    },
                    row.eligible,
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(actual_routes, expected_routes);
        for selection in 0..3 {
            let member = selected_member(&expected, selection);
            let encoded = crate::engine_attestation::placement::candidate(&member);
            let accepted = engine
                .plans
                .values()
                .filter(|candidate| {
                    let crate::engine_operation::StoredPlacements::Product(product) =
                        &candidate.operations[&clear].placements
                    else {
                        panic!("Clear product storage");
                    };
                    product.validate_member(&encoded).is_ok()
                })
                .count();
            assert_eq!(
                accepted, 1,
                "one proof candidate accepts each literal member"
            );
        }
    }
    // Removing one offered relation changes only the facade's relation census.
    let missing = RelationId::new(CLEAR, RelationKind::ClearClamp, RelationSubject::Operation);
    let mut changed = facade;
    assert!(changed.relations_mut().remove(&missing).is_some());
    assert_eq!(
        validate_clear_plan(&analyzed, &changed),
        Err(crate::CompileError::TargetPlanRelationCensusMismatch { relation: missing }),
    );
}

fn bound_scope(operations: &[OperationId]) -> crate::BoundCompilerInput {
    let realization = realization::derive(
        &architecture::ARCHITECTURE,
        realization::RealizationScope::from_operations(operations.iter().copied())
            .expect("realization scope"),
    )
    .expect("complete realization");
    crate::bind_input(
        &architecture::ARCHITECTURE,
        realization,
        crate::CompilationScope::from_operations(operations.iter().copied())
            .expect("compiler scope"),
        super::test_policy(),
    )
    .expect("public bound scope")
}

fn union_reasons(
    left: &BTreeMap<realization::FactId, BTreeSet<realization::DisclosureReason>>,
    right: &BTreeMap<realization::FactId, BTreeSet<realization::DisclosureReason>>,
) -> BTreeMap<realization::FactId, BTreeSet<realization::DisclosureReason>> {
    let mut union = left.clone();
    for (fact, reasons) in right {
        union
            .entry(fact.clone())
            .or_default()
            .extend(reasons.iter().cloned());
    }
    union
}

fn assert_reason_union(
    left: &crate::BoundCompilerInput,
    right: &crate::BoundCompilerInput,
    mixed: &crate::BoundCompilerInput,
) {
    let left = left.realization().project().declassification;
    let right = right.realization().project().declassification;
    let mixed = mixed.realization().project().declassification;
    assert_eq!(
        mixed.required_public,
        union_reasons(&left.required_public, &right.required_public),
    );
    assert_eq!(
        mixed.newly_disclosed,
        union_reasons(&left.newly_disclosed, &right.newly_disclosed),
    );
    assert_eq!(
        mixed.retained_private,
        left.retained_private
            .union(&right.retained_private)
            .cloned()
            .collect(),
    );
}

fn assert_compact_local(
    standalone: &crate::operation_plan::ValidatedTargetOperationPlan,
    mixed: &crate::operation_plan::ValidatedTargetOperationPlan,
) {
    assert_eq!(standalone.representation(), mixed.representation());
    assert_eq!(
        standalone.cases().collect::<Vec<_>>(),
        mixed.cases().collect::<Vec<_>>()
    );
    assert_partner_relations(standalone.relations(), mixed.relations(), CLEAR);
    assert_eq!(
        standalone.carriers().collect::<Vec<_>>(),
        mixed.carriers().collect::<Vec<_>>()
    );
    assert_eq!(
        standalone.layout().collect::<Vec<_>>(),
        mixed.layout().collect::<Vec<_>>()
    );
    assert_eq!(
        standalone.coverage().collect::<Vec<_>>(),
        mixed.coverage().collect::<Vec<_>>()
    );
    assert_eq!(
        standalone.capabilities().collect::<Vec<_>>(),
        mixed.capabilities().collect::<Vec<_>>()
    );
    assert_eq!(
        standalone.external_evidence().collect::<Vec<_>>(),
        mixed.external_evidence().collect::<Vec<_>>()
    );
}

fn assert_announcement_local(
    standalone: &crate::maturity_announcement_plan::ValidatedMaturityAnnouncementOperationPlan,
    mixed: &crate::maturity_announcement_plan::ValidatedMaturityAnnouncementOperationPlan,
) {
    use crate::maturity_announcement_plan::MaturityAnnouncementRepresentationPlan as Mode;
    assert_eq!(standalone.representation(), mixed.representation());
    assert_eq!(standalone.sponsor(), mixed.sponsor());
    assert_eq!(standalone.state(), mixed.state());
    assert_eq!(standalone.operator(), mixed.operator());
    assert_eq!(standalone.canonical(), mixed.canonical());
    assert_eq!(standalone.roots(), mixed.roots());
    assert_eq!(standalone.certificate(), mixed.certificate());
    assert_eq!(standalone.constructibility(), mixed.constructibility());
    assert_eq!(standalone.public_facts(), mixed.public_facts());
    assert_eq!(standalone.transition(), mixed.transition());
    assert_eq!(standalone.root_history(), mixed.root_history());
    assert_eq!(standalone.public_recovery(), mixed.public_recovery());
    for mode in [Mode::Explicit, Mode::PublicCommitted] {
        let left = standalone.projection(mode).expect("standalone mode");
        let right = mixed.projection(mode).expect("mixed mode");
        assert_eq!(
            left.cases().collect::<Vec<_>>(),
            right.cases().collect::<Vec<_>>()
        );
        assert_partner_relations(left.relations(), right.relations(), CLEAR);
        assert_eq!(
            left.carriers().collect::<Vec<_>>(),
            right.carriers().collect::<Vec<_>>()
        );
        assert_eq!(
            left.layout().collect::<Vec<_>>(),
            right.layout().collect::<Vec<_>>()
        );
        assert_eq!(
            left.coverage().collect::<Vec<_>>(),
            right.coverage().collect::<Vec<_>>()
        );
        assert_eq!(
            left.capabilities().collect::<Vec<_>>(),
            right.capabilities().collect::<Vec<_>>()
        );
        assert_eq!(
            left.external_evidence().collect::<Vec<_>>(),
            right.external_evidence().collect::<Vec<_>>()
        );
    }
}

fn assert_clear_lifecycle(
    plan: &crate::clear_operation_plan::ValidatedClearOperationPlan,
    partner: Option<OperationId>,
) {
    use crate::lifecycle::{LifecycleExitStatus, LifecycleRequirement};
    let mut implemented = BTreeSet::from([CLEAR]);
    if let Some(operation) = partner {
        implemented.insert(operation);
    }
    assert_eq!(
        plan.lifecycle().implemented().collect::<BTreeSet<_>>(),
        implemented
    );
    let outstanding = [
        (ObjectId::Ash, OperationId::CompactAsh),
        (ObjectId::State, OperationId::AdmitDeposits),
        (ObjectId::State, OperationId::Cycle),
        (ObjectId::State, OperationId::Redeem),
        (ObjectId::State, OperationId::ReceiptRelabel),
        (ObjectId::State, OperationId::AnnounceMaturity),
    ]
    .into_iter()
    .filter(|(_, exit)| Some(*exit) != partner)
    .map(|(object, exit)| LifecycleRequirement {
        object,
        representation: RepresentationMode::Explicit,
        exit,
        status: LifecycleExitStatus::DeclaredOutsideCompilerScope,
    })
    .collect::<BTreeSet<_>>();
    assert_eq!(
        plan.lifecycle()
            .outstanding()
            .cloned()
            .collect::<BTreeSet<_>>(),
        outstanding
    );
    assert!(!plan.lifecycle().release_complete());
}

fn assert_clear_local(
    standalone: &crate::clear_operation_plan::ValidatedClearOperationPlan,
    mixed: &crate::clear_operation_plan::ValidatedClearOperationPlan,
    partner: OperationId,
) {
    assert_eq!(standalone.representation(), mixed.representation());
    assert_eq!(
        standalone.fields().collect::<Vec<_>>(),
        mixed.fields().collect::<Vec<_>>()
    );
    assert_eq!(
        standalone.cases().collect::<Vec<_>>(),
        mixed.cases().collect::<Vec<_>>()
    );
    assert_partner_relations(standalone.relations(), mixed.relations(), partner);
    assert_eq!(
        standalone.carriers().collect::<Vec<_>>(),
        mixed.carriers().collect::<Vec<_>>()
    );
    assert_eq!(
        standalone.layout().collect::<Vec<_>>(),
        mixed.layout().collect::<Vec<_>>()
    );
    assert_eq!(
        standalone.coverage().collect::<Vec<_>>(),
        mixed.coverage().collect::<Vec<_>>()
    );
    assert_eq!(
        standalone.capabilities().collect::<Vec<_>>(),
        mixed.capabilities().collect::<Vec<_>>()
    );
    assert_eq!(
        standalone.external_evidence().collect::<Vec<_>>(),
        mixed.external_evidence().collect::<Vec<_>>()
    );
    assert_clear_lifecycle(standalone, None);
    assert_clear_lifecycle(mixed, Some(partner));
}

fn assert_partner_relations<'a>(
    standalone: impl Iterator<Item = &'a crate::operation_plan::TargetRelationRequirement>,
    mixed: impl Iterator<Item = &'a crate::operation_plan::TargetRelationRequirement>,
    partner: OperationId,
) {
    use crate::lifecycle::LifecycleExitStatus;
    let expected_rows = standalone.collect::<Vec<_>>();
    let actual = mixed
        .map(|row| (row.relation.clone(), row))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(expected_rows.len(), actual.len());
    for row in expected_rows {
        let mut expected = row.clone();
        if matches!(
            expected.relation.subject(),
            RelationSubject::LifecycleExit { exit, .. } if *exit == partner
        ) {
            assert_eq!(expected.lifecycle.len(), 1);
            assert!(
                expected
                    .lifecycle
                    .iter()
                    .all(|item| item.status == LifecycleExitStatus::DeclaredOutsideCompilerScope)
            );
            expected.lifecycle = expected
                .lifecycle
                .into_iter()
                .map(|mut item| {
                    item.status = LifecycleExitStatus::AvailableInCompilerScope;
                    item
                })
                .collect();
        }
        assert_eq!(Some(&expected), actual.get(&row.relation).copied());
    }
}

#[test]
fn clear_scopes_preserve_local_requirements() {
    use crate::{
        maturity_announcement_plan::plan_maturity_announcement_target_operation,
        operation_plan::plan_compact_ash_target_operation,
    };

    let clear = bound_scope(&[CLEAR]);
    let standalone_clear =
        crate::clear_operation_plan::plan_clear_target_operation(&clear, limits())
            .expect("standalone Clear");
    let compact = bound_scope(&[OperationId::CompactAsh]);
    let compact_clear = bound_scope(&[OperationId::CompactAsh, CLEAR]);
    let standalone_compact =
        plan_compact_ash_target_operation(&compact, limits()).expect("standalone compact");
    let mixed_compact =
        plan_compact_ash_target_operation(&compact_clear, limits()).expect("compact with Clear");
    let clear_compact =
        crate::clear_operation_plan::plan_clear_target_operation(&compact_clear, limits())
            .expect("Clear with compact");
    assert_eq!(clear_compact.operation(), CLEAR);
    assert_clear_local(&standalone_clear, &clear_compact, OperationId::CompactAsh);
    assert_compact_local(&standalone_compact, &mixed_compact);
    assert_eq!(standalone_compact.lifecycle().implemented().count(), 1);
    assert_eq!(standalone_compact.lifecycle().outstanding().count(), 1);
    assert!(!standalone_compact.lifecycle().release_complete());
    assert_eq!(
        mixed_compact
            .lifecycle()
            .implemented()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([OperationId::CompactAsh, CLEAR])
    );
    assert_eq!(mixed_compact.lifecycle().outstanding().count(), 0);
    assert!(mixed_compact.lifecycle().release_complete());
    assert_reason_union(&compact, &clear, &compact_clear);
    let exit = RelationId::new(
        OperationId::CompactAsh,
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit {
            object: ObjectId::Ash,
            exit: CLEAR,
        },
    );
    let mut stale_exit = mixed_compact;
    stale_exit
        .relations_mut()
        .get_mut(&exit)
        .expect("mixed Clear exit row")
        .lifecycle = standalone_compact
        .relation(&exit)
        .expect("standalone Clear exit row")
        .lifecycle
        .clone();
    let analyzed_mixed =
        analyze_scoped_program(&compact_clear, limits()).expect("complete mixed analysis");
    assert_eq!(
        crate::operation_plan::validate_target_operation_plan(&analyzed_mixed, &stale_exit),
        Err(crate::CompileError::TargetPlanRelationRequirementMismatch),
    );

    let announcement = bound_scope(&[OperationId::AnnounceMaturity]);
    let announcement_clear = bound_scope(&[OperationId::AnnounceMaturity, CLEAR]);
    let standalone_announcement =
        plan_maturity_announcement_target_operation(&announcement, limits())
            .expect("standalone announcement");
    let mixed_announcement =
        plan_maturity_announcement_target_operation(&announcement_clear, limits())
            .expect("announcement with Clear");
    let clear_announcement =
        crate::clear_operation_plan::plan_clear_target_operation(&announcement_clear, limits())
            .expect("Clear with announcement");
    assert_eq!(clear_announcement.operation(), CLEAR);
    assert_clear_local(
        &standalone_clear,
        &clear_announcement,
        OperationId::AnnounceMaturity,
    );
    assert_announcement_local(&standalone_announcement, &mixed_announcement);
    assert_eq!(standalone_announcement.lifecycle().implemented().count(), 1);
    assert_eq!(standalone_announcement.lifecycle().outstanding().count(), 5);
    assert_eq!(
        mixed_announcement
            .lifecycle()
            .implemented()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([OperationId::AnnounceMaturity, CLEAR])
    );
    assert_eq!(mixed_announcement.lifecycle().outstanding().count(), 4);
    assert_reason_union(&announcement, &clear, &announcement_clear);
}
