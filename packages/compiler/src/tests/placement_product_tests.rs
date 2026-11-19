//! Literal small-product controls for the checked placement representation.

use std::collections::BTreeSet;
use std::num::NonZeroU64;

use architecture::semantic::SemanticProfileId;

use crate::{
    carrier::CarrierQuantification,
    engine_identity::{self, CarrierKey, CaseKey, LayoutKey},
    engine_placement::{self as placement, Assignment, Candidate, EmptyCause, Entry, Error, Key},
    engine_records::{OperationKey, RelationKey},
    placement::{CarrierMultiplicity, PlacementSearchLimits, PlacementSearchReport},
};

fn compare_scoped_storage(records: &crate::engine_program::Records, policy: crate::AnalysisPolicy) {
    use crate::{engine_operation::StoredPlacements, engine_program as program};
    let placement_limits = limits(10_000_000, 1_000_000);
    let old = program::analyze(records, policy.proof_search_limits, placement_limits)
        .expect("enumerated control");
    let operation_keys = old
        .plans
        .values()
        .flat_map(|plan| plan.operations.keys().copied())
        .collect::<BTreeSet<_>>();
    let product = program::analyze_prepared_scoped(
        records,
        policy.proof_search_limits,
        placement_limits,
        None,
        &operation_keys,
    )
    .expect("checked products");
    assert_eq!(old.source, product.source);
    assert_eq!(old.relations, product.relations);
    assert_eq!(old.predicates, product.predicates);
    assert_eq!(old.sources, product.sources);
    assert_eq!(old.regions, product.regions);
    assert_eq!(old.constructibility, product.constructibility);
    assert_eq!(old.construction, product.construction);
    assert_eq!(old.lifecycle, product.lifecycle);
    assert_eq!(old.evidence, product.evidence);
    assert_eq!(old.proof_search, product.proof_search);
    assert_eq!(
        old.plans.keys().collect::<Vec<_>>(),
        product.plans.keys().collect::<Vec<_>>()
    );
    for (key, old_plan) in &old.plans {
        let new_plan = &product.plans[key];
        assert_eq!(old_plan.requirements, new_plan.requirements);
        assert_eq!(
            old_plan.operations.keys().collect::<Vec<_>>(),
            new_plan.operations.keys().collect::<Vec<_>>()
        );
        for (operation, old_factor) in &old_plan.operations {
            let new_factor = &new_plan.operations[operation];
            assert_eq!(old_factor.id, new_factor.id);
            assert_eq!(old_factor.cases, new_factor.cases);
            assert_eq!(old_factor.bundles, new_factor.bundles);
            assert_eq!(old_factor.layout, new_factor.layout);
            assert_eq!(old_factor.coverage, new_factor.coverage);
            let StoredPlacements::Product(checked) = &new_factor.placements else {
                panic!("requested product storage for every operation");
            };
            let mut report = PlacementSearchReport::default();
            let expanded = checked
                .expand(placement_limits, &mut report)
                .expect("bounded real factor");
            assert_eq!(old_factor.placements, expanded, "{key:?} {operation:?}");
        }
    }
}

fn active_facade_figures() {
    use crate::{
        live_transfer_plan::{
            LiveTransferRepresentationPlan as L, plan_live_transfer_target_operation,
        },
        maturity_announcement_plan::{
            MaturityAnnouncementRepresentationPlan as A,
            plan_maturity_announcement_target_operation,
        },
    };
    use architecture::OperationId;
    let search = limits(10_000_000, 1_000_000);
    let compact = super::bound_input(&[OperationId::CompactAsh]);
    let facade = crate::operation_plan::plan_compact_ash_target_operation(&compact, search)
        .expect("public compact facade");
    assert_eq!(
        (
            facade.relations().count(),
            facade.cases().count(),
            facade.layout().count()
        ),
        (23, 2, 63)
    );
    let live = super::bound_input(&[OperationId::TransferLive]);
    let facade = plan_live_transfer_target_operation(&live, search).expect("public live facade");
    for (mode, expected_layout, expected_coverage) in
        [(L::Explicit, 63, 221), (L::PrivateCommitted, 55, 223)]
    {
        let projection = facade.projection(mode).expect("live representation");
        assert_eq!(
            (
                projection.relations().count(),
                projection.cases().count(),
                projection.layout().count(),
                projection.coverage().count()
            ),
            (24, 2, expected_layout, expected_coverage),
        );
    }
    let announcement = super::announcement_input();
    let facade = plan_maturity_announcement_target_operation(&announcement, search)
        .expect("public announcement facade");
    for mode in [A::Explicit, A::PublicCommitted] {
        let projection = facade
            .projection(mode)
            .expect("announcement representation");
        assert_eq!(
            (
                projection.relations().count(),
                projection.cases().count(),
                projection.layout().count()
            ),
            (26, 2, 59)
        );
    }
    let burn = super::source_tests::burn_control().input;
    let facade = crate::burn_operation_plan::plan_burn_target_operation(&burn, search)
        .expect("public Burn facade");
    assert_eq!(
        (facade.relations().count(), facade.cases().count()),
        (32, 2)
    );
}

fn deleted_clear_option_is_refused(
    input: &crate::BoundCompilerInput,
    analysis: &crate::analyzed::ScopedAnalyzedProgram,
    search: PlacementSearchLimits,
) {
    let mut corrupted = analysis.clone();
    let factor = corrupted
        .proof_plans
        .values_mut()
        .next()
        .expect("first Clear plan")
        .operations
        .get_mut(&architecture::OperationId::Clear)
        .expect("Clear factor");
    let retained_coverage = factor.coverage.clone();
    let product = factor
        .feasible_placements
        .product_mut()
        .expect("Clear product");
    let options = product
        .factors
        .values_mut()
        .next()
        .expect("first runtime factor");
    let dropped = options.iter().next().expect("local option").clone();
    options.remove(&dropped);
    assert_eq!(factor.coverage, retained_coverage);
    assert!(matches!(
        crate::analyzed_validate::validate_scoped_analyzed_program(input, search, &corrupted),
        Err(crate::CompileError::AnalyzedPlacementProductMismatch { .. }),
    ));
}

#[test]
fn placement_storage_preserves_active_plans() {
    use architecture::semantic::{KeyedOperationId, SemanticOperationId};
    use architecture::{
        OperationId,
        keyed::KeyedManifest,
        keyed_fixed::{KeyedFixedManifest, KeyedFixedOperationId},
    };
    for operations in [
        vec![OperationId::CompactAsh],
        vec![OperationId::TransferLive],
    ] {
        let input = super::bound_input(&operations);
        let records = crate::engine_program::lower(&input).expect("attestation records");
        compare_scoped_storage(&records, input.policy());
    }
    let announcement = super::announcement_input();
    compare_scoped_storage(
        &crate::engine_program::lower(&announcement).expect("announcement records"),
        announcement.policy(),
    );
    let burn = super::source_tests::burn_control().input;
    compare_scoped_storage(
        &crate::engine_program::lower(&burn).expect("Burn records"),
        burn.policy(),
    );
    let policy = super::test_policy();
    let keyed_manifest = KeyedManifest::contract().expect("keyed manifest");
    let keyed_source = realization::keyed::KeyedRealizationDraft::derive(&keyed_manifest)
        .expect("keyed draft")
        .validate(&keyed_manifest)
        .expect("keyed realization");
    let keyed = crate::bind_input(
        &keyed_manifest,
        keyed_source,
        [KeyedOperationId::Source, KeyedOperationId::Merge]
            .into_iter()
            .map(SemanticOperationId::Keyed)
            .collect(),
        policy,
    )
    .expect("keyed bound input");
    compare_scoped_storage(
        &crate::engine_program::lower(&keyed).expect("keyed records"),
        keyed.policy(),
    );
    let fixed_manifest = KeyedFixedManifest::contract().expect("fixed manifest");
    let fixed_source =
        realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&fixed_manifest)
            .expect("fixed draft")
            .validate(&fixed_manifest)
            .expect("fixed realization");
    let fixed = crate::bind_input(
        &fixed_manifest,
        fixed_source,
        vec![KeyedFixedOperationId::Source, KeyedFixedOperationId::Merge],
        policy,
    )
    .expect("fixed bound input");
    compare_scoped_storage(
        &crate::engine_program::lower(&fixed).expect("fixed records"),
        fixed.policy(),
    );
    active_facade_figures();
    let clear = super::source_tests::clear_control().input;
    let search = limits(10_000_000, 100_000);
    let complete = crate::analyzed::analyze_scoped_program(&clear, search)
        .expect("complete Clear control for product corruption");
    deleted_clear_option_is_refused(&clear, &complete, search);
}

fn check_clear_engine_products(
    engine: &crate::engine_program::Program<crate::engine_operation::StoredPlacements>,
    clear: OperationKey,
    search: PlacementSearchLimits,
) -> (u64, usize, usize) {
    use crate::engine_operation::StoredPlacements;
    assert_eq!(engine.plans.len(), 4);
    let mut state_requirement = u64::MAX;
    let mut measured_factors = 0_usize;
    let mut measured_options = 0_usize;
    for plan in engine.plans.values() {
        let factor = &plan.operations[&clear];
        let StoredPlacements::Product(product) = &factor.placements else {
            panic!("Clear must retain a product");
        };
        state_requirement = state_requirement.min(factor.search.states_visited);
        measured_factors += product.factors().count();
        measured_options += product
            .factors()
            .map(|(_, options)| options.len())
            .sum::<usize>();
        assert_eq!(product.factors().count(), 56);
        assert_eq!(
            product
                .factors()
                .map(|(_, options)| options.len())
                .sum::<usize>(),
            127
        );
        let option_classes = product.factors().map(|(_, options)| options.len()).fold(
            (0, 0, 0),
            |(two, three, four), count| match count {
                2 => (two + 1, three, four),
                3 => (two, three + 1, four),
                4 => (two, three, four + 1),
                other => panic!("unexpected factor option count {other}"),
            },
        );
        assert_eq!(option_classes, (46, 5, 5));
        assert_eq!(
            product.checked_cardinality(),
            Some(17_509_995_351_216_488_448)
        );
        assert_eq!(
            (
                factor.search.complete_assignments,
                factor.search.feasible_placements
            ),
            (0, 0)
        );
        assert_eq!(
            product.expand(search, &mut PlacementSearchReport::default()),
            Err(Error::CandidateLimit(100_000)),
        );
    }
    assert_eq!((measured_factors, measured_options), (224, 508));
    assert!(state_requirement > 1);
    (state_requirement, measured_factors, measured_options)
}

fn check_clear_literal_factors(
    input: &crate::BoundCompilerInput,
    analysis: &crate::analyzed::ScopedAnalyzedProgram,
    engine: &crate::engine_program::Program<crate::engine_operation::StoredPlacements>,
    clear: OperationKey,
) {
    use crate::engine_operation::StoredPlacements;
    let relations = crate::relation::build_relation_analysis(input).expect("Clear relations");
    assert_eq!(analysis.proof_plans.len(), 4);
    for plan in analysis.proof_plans.values() {
        let product = plan.operations[&architecture::OperationId::Clear]
            .feasible_placements
            .product()
            .expect("typed Clear factors");
        let cases = crate::case::execution_cases(&relations, &plan.proof_plan)
            .expect("independent Clear cases");
        let plans = crate::placement::classify_relation_cases(&relations, &cases)
            .expect("independent Clear relation cases");
        // Carrier choices below are literal; routing availability and family
        // scope come from the same eligibility owner that lowering calls. The
        // test-side layout rule is independent of product construction and
        // layout union; the 3.5b oracle owns independent Clear routing.
        let eligibility = crate::carrier::relation_case_eligibility(&relations, &plans)
            .expect("Clear carrier eligibility");
        let expected = expected_clear_options(literal_clear_carriers(&cases), &eligibility);
        assert_eq!(product, &expected);
        for selection in 0..3 {
            let member = selected_clear_member(&expected, selection);
            let encoded = crate::engine_attestation::placement::candidate(&member);
            let accepted = engine
                .plans
                .values()
                .filter(|candidate| {
                    let StoredPlacements::Product(product) =
                        &candidate.operations[&clear].placements
                    else {
                        panic!("Clear product storage");
                    };
                    product.validate_member(&encoded).is_ok()
                })
                .count();
            assert_eq!(
                accepted, 1,
                "one Clear proof candidate accepts each literal member"
            );
        }
    }
}

#[test]
fn clear_product_analysis_is_complete() {
    use crate::{
        analyzed::analyze_scoped_program, analyzed_validate::validate_scoped_analyzed_program,
        engine_program,
    };
    use architecture::OperationId;
    use std::time::Instant;
    let input = super::source_tests::clear_control().input;
    let generous = limits(10_000_000, 100_000);
    let started = Instant::now();
    let (records, stages) =
        crate::engine_attestation::program::test_preparation(&input).expect("Clear preparation");
    let clear = crate::engine_attestation::operation_key(OperationId::Clear);
    let engine = engine_program::analyze_prepared_scoped(
        &records,
        input.policy().proof_search_limits,
        generous,
        Some(stages),
        &BTreeSet::from([clear]),
    )
    .expect("complete checked Clear products");
    let construction_wall = started.elapsed();
    let (state_requirement, measured_factors, measured_options) =
        check_clear_engine_products(&engine, clear, generous);
    let exact_state = engine_program::analyze_prepared_scoped(
        &records,
        input.policy().proof_search_limits,
        limits(state_requirement, 100_000),
        None,
        &BTreeSet::from([clear]),
    );
    assert!(
        exact_state.is_ok(),
        "the measured construction limit admits Clear"
    );
    let low = limits(state_requirement - 1, 100_000);
    let low_state = engine_program::analyze_prepared_scoped(
        &records,
        input.policy().proof_search_limits,
        low,
        None,
        &BTreeSet::from([clear]),
    );
    assert_eq!(
        low_state.err(),
        Some(engine_program::Error::Operations(Box::new(
            crate::engine_operation::Error::Placement(Box::new(Error::StateLimit(
                state_requirement - 1,
            ))),
        ))),
    );
    let attestation_started = Instant::now();
    let derived = crate::engine_attestation::program::derive(&input, generous)
        .expect("complete attestation construction");
    let attestation_wall = attestation_started.elapsed();
    assert_eq!(derived.plans.len(), 4);
    let full_analysis_started = Instant::now();
    let analysis =
        analyze_scoped_program(&input, generous).expect("complete internal Clear analysis");
    let full_analysis_wall = full_analysis_started.elapsed();
    check_clear_literal_factors(&input, &analysis, &engine, clear);
    let revalidation_started = Instant::now();
    validate_scoped_analyzed_program(&input, generous, &analysis)
        .expect("fresh Clear revalidation");
    let revalidation_wall = revalidation_started.elapsed();
    deleted_clear_option_is_refused(&input, &analysis, generous);
    tracing::info!(
        target: "compiler::analysis",
        core_construction_wall_ms = u64::try_from(construction_wall.as_millis()).unwrap_or(u64::MAX),
        construction_wall_ms = u64::try_from(attestation_wall.as_millis()).unwrap_or(u64::MAX),
        full_analysis_wall_ms = u64::try_from(full_analysis_wall.as_millis()).unwrap_or(u64::MAX),
        revalidation_wall_ms = u64::try_from(revalidation_wall.as_millis()).unwrap_or(u64::MAX),
        total_wall_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        factor_count = u64::try_from(measured_factors).unwrap_or(u64::MAX),
        option_count = u64::try_from(measured_options).unwrap_or(u64::MAX),
        minimum_product_state_requirement = state_requirement,
        represented_per_candidate = 17_509_995_351_216_488_448_u64,
        message = "clear product control completed",
    );
}

fn key(code: u16) -> Key {
    Key {
        relation: RelationKey {
            operation: OperationKey {
                namespace: 1,
                code: 1,
            },
            kind: code,
            subject: vec![code],
        },
        case: CaseKey::new(
            SemanticProfileId::Attestation,
            vec![engine_identity::word(1_u16)],
        ),
    }
}

fn placed(code: u16, quantification: CarrierQuantification) -> placement::Placed {
    placement::Placed {
        carrier: CarrierKey::new(
            SemanticProfileId::Attestation,
            vec![engine_identity::word(code)],
        ),
        quantification,
    }
}

fn layout(code: u16) -> LayoutKey {
    LayoutKey::new(
        SemanticProfileId::Attestation,
        vec![engine_identity::word(code)],
    )
}

fn entry(code: u16, quantification: CarrierQuantification, layout_codes: &[u16]) -> Entry {
    Entry {
        placed: placed(code, quantification),
        layout: layout_codes.iter().map(|code| layout(*code)).collect(),
    }
}

fn row(
    code: u16,
    multiplicity: CarrierMultiplicity,
    entries: Vec<Entry>,
) -> placement::Eligibility {
    placement::Eligibility {
        key: key(code),
        multiplicity,
        options: entries.clone(),
        entries,
        empty_cause: EmptyCause::NoEligible,
    }
}

fn control() -> placement::Records {
    use CarrierQuantification::{CompleteFamilyProof, PerMember, Single};
    let eligibility = vec![
        row(
            1,
            CarrierMultiplicity::ExactlyOne,
            vec![entry(11, Single, &[91]), entry(12, Single, &[])],
        ),
        row(
            2,
            CarrierMultiplicity::AtLeastOne,
            vec![entry(21, Single, &[91]), entry(22, Single, &[92])],
        ),
        row(
            3,
            CarrierMultiplicity::EveryMember,
            vec![
                entry(31, PerMember, &[93]),
                entry(32, CompleteFamilyProof, &[93]),
            ],
        ),
        row(
            4,
            CarrierMultiplicity::DeliberateDuplication,
            vec![entry(41, Single, &[94]), entry(42, Single, &[])],
        ),
    ];
    placement::Records {
        plans: eligibility
            .iter()
            .map(|row| (row.key.clone(), true))
            .collect(),
        eligibility,
    }
}

fn limits(states: u64, candidates: u64) -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        NonZeroU64::new(states).expect("positive state budget"),
        NonZeroU64::new(candidates).expect("positive candidate budget"),
    )
}

fn clear_placed(
    carrier: crate::carrier::CarrierRole,
    quantification: CarrierQuantification,
) -> crate::placement::PlacedCarrier {
    crate::placement::PlacedCarrier {
        carrier,
        quantification,
    }
}

fn literal_clear_factor(
    table: &mut std::collections::BTreeMap<
        crate::placement::RelationCaseKey,
        BTreeSet<crate::placement::PlacedCarrier>,
    >,
    case: &crate::case::ExecutionCaseId,
    kind: realization::RelationKind,
    subject: realization::RelationSubject,
    choices: impl IntoIterator<Item = crate::placement::PlacedCarrier>,
) {
    let key = crate::placement::RelationCaseKey {
        relation: realization::RelationId::new(architecture::OperationId::Clear, kind, subject),
        case: case.clone(),
    };
    assert!(table.insert(key, choices.into_iter().collect()).is_none());
}

fn literal_clear_carriers(
    cases: &[crate::case::ExecutionCase],
) -> std::collections::BTreeMap<
    crate::placement::RelationCaseKey,
    BTreeSet<crate::placement::PlacedCarrier>,
> {
    use crate::carrier::CarrierRole as C;
    use CarrierQuantification::{CompleteFamilyProof, PerMember, Single};
    use architecture::{ObjectId as O, OperationId};
    use realization::{RelationKind as K, RelationSubject as S, StateField, TransactionSide as T};
    let globals = |quantification| {
        [O::State, O::Ash].map(|anchor| {
            clear_placed(
                C::OperationGlobal {
                    operation: OperationId::Clear,
                    anchor,
                },
                quantification,
            )
        })
    };
    let mut table = std::collections::BTreeMap::new();
    for case in cases {
        for side in [T::Input, T::Output] {
            for object in [O::State, O::Ash, O::PlainLbtc] {
                if object == O::PlainLbtc && case.id.sponsor == crate::case::SponsorCase::Absent {
                    continue;
                }
                let subject = S::ObjectFamily { side, object };
                let mut cardinality = globals(Single).to_vec();
                let mut recognition = if side == T::Input {
                    globals(CompleteFamilyProof).to_vec()
                } else {
                    globals(Single).to_vec()
                };
                if side == T::Input {
                    cardinality.push(clear_placed(C::InputFamilyCoordinator { object }, Single));
                    recognition.push(clear_placed(
                        C::InputFamilyCoordinator { object },
                        CompleteFamilyProof,
                    ));
                    recognition.push(clear_placed(
                        C::EveryInputFamilyMember { object },
                        PerMember,
                    ));
                }
                literal_clear_factor(
                    &mut table,
                    &case.id,
                    K::Cardinality,
                    subject.clone(),
                    cardinality,
                );
                literal_clear_factor(&mut table, &case.id, K::Recognition, subject, recognition);
            }
            literal_clear_factor(
                &mut table,
                &case.id,
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
            literal_clear_factor(&mut table, &case.id, kind, subject, globals(Single));
        }
        for field in StateField::ALL {
            literal_clear_factor(
                &mut table,
                &case.id,
                K::ClearAssignment,
                S::StateField { field: *field },
                globals(Single),
            );
        }
    }
    assert_eq!(table.len(), 56);
    table
}

fn expected_clear_local_layout(
    analysis: &crate::carrier::CarrierEligibility,
    placed: &crate::placement::PlacedCarrier,
) -> BTreeSet<crate::layout::LayoutRequirement> {
    use crate::{
        carrier::{CarrierAvailability, CarrierRole},
        layout::LayoutRequirement as L,
        placement::SemanticScope,
    };
    let entry = analysis
        .eligible
        .iter()
        .find(|entry| {
            entry.carrier == placed.carrier && entry.quantification == placed.quantification
        })
        .expect("literal carrier has eligibility entry");
    let mut layout = BTreeSet::new();
    if matches!(
        placed.carrier,
        CarrierRole::OperationGlobal { .. } | CarrierRole::InputFamilyCoordinator { .. }
    ) {
        layout.insert(L::CanonicalCoordinator {
            operation: architecture::OperationId::Clear,
            anchor: placed.carrier.clone(),
        });
    }
    if placed.quantification == CarrierQuantification::CompleteFamilyProof
        && let SemanticScope::MemberLocal { side, object } = analysis.scope
    {
        layout.insert(L::AuthenticateFamilyCensus {
            relation: analysis.relation.clone(),
            side,
            object,
        });
        layout.insert(L::CompleteAndDisjointFamilies {
            relation: analysis.relation.clone(),
            side,
        });
    }
    for routing in &entry.routings {
        match routing.availability {
            CarrierAvailability::Intrinsic => {}
            CarrierAvailability::LayoutProvided => {
                layout.insert(L::MakeSourceAvailable {
                    relation: analysis.relation.clone(),
                    case: analysis.case.clone(),
                    carrier: placed.carrier.clone(),
                    source: routing.source.clone(),
                });
            }
            CarrierAvailability::SponsorRegionConfined => {
                layout.insert(L::IsolateSponsorRegion {
                    relation: analysis.relation.clone(),
                    case: analysis.case.clone(),
                });
            }
        }
    }
    layout
}

fn expected_clear_options(
    carriers: std::collections::BTreeMap<
        crate::placement::RelationCaseKey,
        BTreeSet<crate::placement::PlacedCarrier>,
    >,
    eligibility: &[crate::carrier::CarrierEligibility],
) -> crate::analyzed_operation::PlacementProduct {
    let factors = carriers
        .into_iter()
        .map(|(key, choices)| {
            let analysis = eligibility
                .iter()
                .find(|row| row.relation == key.relation && row.case == key.case)
                .expect("literal runtime factor has eligibility row");
            let options = choices
                .into_iter()
                .map(|placed| crate::analyzed_operation::PlacementFactorOption {
                    layout_requirements: expected_clear_local_layout(analysis, &placed),
                    carriers: BTreeSet::from([placed]),
                })
                .collect();
            (key, options)
        })
        .collect();
    crate::analyzed_operation::PlacementProduct { factors }
}

fn selected_clear_member(
    table: &crate::analyzed_operation::PlacementProduct,
    selection: usize,
) -> crate::placement::PlacementCandidate {
    let mut assignments = Vec::new();
    let mut layout_requirements = BTreeSet::new();
    for (index, (key, options)) in table.factors.iter().enumerate() {
        let option = match selection {
            0 => options.first().expect("literal first option"),
            1 => options.last().expect("literal last option"),
            _ if index.is_multiple_of(2) => options.first().expect("literal mixed option"),
            _ => options.last().expect("literal mixed option"),
        };
        assignments.push(crate::placement::PlacementAssignment {
            relation: key.relation.clone(),
            case: key.case.clone(),
            carriers: option.carriers.iter().cloned().collect(),
        });
        layout_requirements.extend(option.layout_requirements.iter().cloned());
    }
    crate::placement::PlacementCandidate {
        assignments,
        layout_requirements: layout_requirements.into_iter().collect(),
    }
}

fn expected_members() -> Vec<Candidate> {
    use CarrierQuantification::{CompleteFamilyProof, PerMember, Single};
    // This literal table and its loops form expectations without calling the
    // production option builder or reading factors back from the product.
    let first = [(11, Single, &[91_u16][..]), (12, Single, &[])];
    let second = [(21, Single, &[91_u16][..]), (22, Single, &[92_u16][..])];
    let third = [
        (31, PerMember, &[93_u16][..]),
        (32, CompleteFamilyProof, &[93_u16][..]),
    ];
    let fourth = [(41, Single, &[94_u16][..]), (42, Single, &[])];
    let mut expected = Vec::new();
    for a in first {
        for b in second {
            for c in third {
                let mut local_layout = BTreeSet::new();
                for code in a.2.iter().chain(b.2).chain(c.2).chain(fourth[0].2) {
                    local_layout.insert(layout(*code));
                }
                expected.push(Candidate {
                    assignments: vec![
                        Assignment {
                            key: key(1),
                            carriers: vec![placed(a.0, a.1)],
                        },
                        Assignment {
                            key: key(2),
                            carriers: vec![placed(b.0, b.1)],
                        },
                        Assignment {
                            key: key(3),
                            carriers: vec![placed(c.0, c.1)],
                        },
                        Assignment {
                            key: key(4),
                            carriers: fourth
                                .iter()
                                .map(|choice| placed(choice.0, choice.1))
                                .collect(),
                        },
                    ],
                    layout: local_layout.into_iter().collect(),
                });
            }
        }
    }
    expected.sort();
    expected
}

#[test]
fn placement_product_matches_exhaustive_oracle() {
    let records = control();
    let mut construction = PlacementSearchReport::default();
    let product = placement::product(&records, limits(100, 8), &mut construction)
        .expect("coherent literal factors");
    let expected = expected_members();
    assert_eq!(expected.len(), 8);
    assert!(expected.len() <= 4_096);
    assert_eq!(product.checked_cardinality(), Some(8));
    assert_eq!(
        product
            .factors()
            .map(|(_, options)| options.len())
            .collect::<Vec<_>>(),
        vec![2, 2, 2, 1]
    );
    assert_eq!(
        (construction.states_visited, construction.retained_options),
        (12, 7)
    );
    assert_eq!(
        (
            construction.complete_assignments,
            construction.feasible_placements
        ),
        (0, 0)
    );
    let mut expansion = PlacementSearchReport::default();
    let expanded = product
        .expand(limits(100, 8), &mut expansion)
        .expect("bounded expansion");
    let mut enumeration = PlacementSearchReport::default();
    let old = placement::enumerate(&records, limits(100, 8), &mut enumeration)
        .expect("old exhaustive witness");
    assert_eq!(expanded, expected);
    assert_eq!(old, expected);
    for member in &expected {
        assert_eq!(product.validate_member(member), Ok(()));
    }
    assert_eq!(
        product.expand(limits(100, 7), &mut PlacementSearchReport::default()),
        Err(Error::CandidateLimit(7))
    );

    let empty = placement::Records {
        plans: Vec::new(),
        eligibility: Vec::new(),
    };
    let zero = placement::product(&empty, limits(1, 1), &mut PlacementSearchReport::default())
        .expect("zero runtime factors");
    assert_eq!(zero.checked_cardinality(), Some(1));
    assert_eq!(
        zero.expand(limits(1, 1), &mut PlacementSearchReport::default()),
        Ok(vec![Candidate {
            assignments: Vec::new(),
            layout: Vec::new()
        }])
    );

    let mut quantification = expected[0].clone();
    quantification.assignments[2].carriers[0].quantification = CarrierQuantification::Single;
    assert!(matches!(
        product.validate_member(&quantification),
        Err(Error::UnpermittedCarrierPlacement { .. })
    ));
    let mut missing = expected[0].clone();
    missing
        .layout
        .retain(|requirement| *requirement != layout(91));
    assert!(matches!(
        product.validate_member(&missing),
        Err(Error::MissingLayoutRequirement(_))
    ));
    let mut surplus = expected[0].clone();
    surplus.layout.push(layout(99));
    assert_eq!(
        product.validate_member(&surplus),
        Err(Error::UnexpectedLayoutRequirement(vec![layout(99)]))
    );
}

#[test]
fn placement_product_rejects_incoherent_factors() {
    let records = control();
    let mut duplicated = records.clone();
    let repeated = duplicated.eligibility[0].options[0].clone();
    duplicated.eligibility[0].options.push(repeated);
    let normalized = placement::product(
        &duplicated,
        limits(100, 8),
        &mut PlacementSearchReport::default(),
    )
    .expect("duplicate local option normalizes");
    assert_eq!(
        normalized
            .factors()
            .next()
            .map(|(_, options)| options.len()),
        Some(2)
    );
    assert_eq!(normalized.checked_cardinality(), Some(8));

    // Another factor also supplies layout 91, so a whole-member union could
    // mask this changed local option. Product admission checks it locally.
    let mut incoherent = records.clone();
    incoherent.eligibility[0].options[0].layout.clear();
    assert_eq!(
        placement::product(
            &incoherent,
            limits(100, 8),
            &mut PlacementSearchReport::default()
        ),
        Err(Error::MissingLayoutRequirement(key(1)))
    );

    let mut missing = records;
    missing.eligibility.pop();
    assert!(matches!(
        placement::product(
            &missing,
            limits(100, 8),
            &mut PlacementSearchReport::default()
        ),
        Err(Error::PlacementCensusMismatch { .. })
    ));
}
