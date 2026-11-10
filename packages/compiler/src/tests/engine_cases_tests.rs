//! Candidate selection never calls the semantic owner back.
use crate::{capability::CapabilityView, engine_cases as core, engine_records::RelationOwner};
use architecture::{
    OperationId,
    keyed::KeyedManifest,
    semantic::{KeyedOperationId, SemanticOperationId},
};

fn keyed() -> crate::BoundCompilerInput<crate::input::KeyedOwner> {
    keyed_scope(&[KeyedOperationId::Source, KeyedOperationId::Merge])
}

#[test]
fn keyed_reverse_registries_restore_complete_identities_and_refuse_missing_entries() {
    let input = keyed();
    let mut lowered = input.owner().lower_relations().expect("accepted lowering");
    let tables = crate::input::KeyedOwner::lower_cases(&input).expect("cases");
    for case in &tables.cases {
        let value = lowered.identities.case(&case.id).expect("registered case");
        assert_eq!(
            crate::engine_keyed::keyed_case(crate::engine_keyed::operation_key(value)),
            case.id
        );
    }
    let placements = crate::engine_keyed::lower_placements(input.owner());
    for plan in crate::engine_keyed::lower_coverage(input.owner()) {
        for row in plan.positive.iter().chain(&plan.negative) {
            let crate::engine_coverage::Purpose::Owner(key) = &row.purpose else {
                panic!("owner purpose before joins")
            };
            let value = lowered.identities.purpose(key).expect("registered purpose");
            assert_eq!(crate::engine_keyed::keyed_purpose(&value), *key);
        }
    }
    for entry in placements.eligibility.iter().flat_map(|row| &row.entries) {
        let (operation, object) = lowered
            .identities
            .carrier(&entry.placed.carrier)
            .expect("registered carrier");
        let [namespace, code] = crate::engine_identity::object(object);
        assert_eq!(
            crate::engine_keyed::keyed_carrier(
                crate::engine_keyed::operation_key(operation),
                crate::engine_lifecycle::ObjectKey { namespace, code }
            ),
            entry.placed.carrier
        );
    }
    let case = tables.cases[0].id.clone();
    let purpose =
        crate::engine_keyed::keyed_purpose(&crate::engine_keyed::KeyedPurpose::ClauseAccepted);
    lowered.identities.purposes.remove(&purpose);
    assert_eq!(
        lowered.identities.purpose(&purpose),
        Err(crate::CompileError::MissingOwnerIdentity)
    );
    lowered.identities.cases.remove(&case);
    assert_eq!(
        lowered.identities.case(&case),
        Err(crate::CompileError::MissingOwnerIdentity)
    );
    let carrier = placements.eligibility[0].entries[0].placed.carrier.clone();
    lowered.identities.carriers.remove(&carrier);
    assert_eq!(
        lowered.identities.carrier(&carrier),
        Err(crate::CompileError::MissingOwnerIdentity)
    );
}

fn keyed_scope(scope: &[KeyedOperationId]) -> crate::BoundCompilerInput<crate::input::KeyedOwner> {
    let manifest = KeyedManifest::contract().expect("manifest");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("declarations")
        .validate(&manifest)
        .expect("validation");
    crate::bind_input(
        &manifest,
        source,
        scope
            .iter()
            .copied()
            .map(SemanticOperationId::Keyed)
            .collect(),
        super::bound_input(&[OperationId::CompactAsh]).policy(),
    )
    .expect("binding")
}
#[test]
fn keyed_case_tables_select_all_evidence_and_runtime_rows() {
    let input = keyed();
    let tables = crate::input::KeyedOwner::lower_cases(&input).expect("tables before search");
    assert_eq!(tables.cases.len(), 2);
    assert_eq!(tables.rows.len(), 9);
    let (proofs, _) =
        crate::engine_records::analyze_bound_proofs(&input, &CapabilityView::Unconstrained)
            .expect("proof");
    let selected = core::select(&tables, &proofs[0]).expect("select");
    assert_eq!(selected.placement.plans.len(), 9);
    assert_eq!(selected.placement.eligibility.len(), 2);
    assert_eq!(selected.coverage.len(), 9);
    let limits = crate::placement::PlacementSearchLimits::new(
        std::num::NonZeroU64::new(100).expect("states"),
        std::num::NonZeroU64::new(10).expect("candidates"),
    );
    let operations =
        crate::engine_operation::analyze(&selected, &[], limits).expect("operation assembly");
    assert_eq!(operations.len(), 2);
    assert_eq!(
        operations
            .values()
            .map(|operation| operation.bundles.len())
            .sum::<usize>(),
        9
    );
    assert_eq!(
        operations
            .values()
            .map(|operation| operation.coverage.nodes.len())
            .sum::<usize>(),
        61
    );
    assert_eq!(
        operations
            .values()
            .map(|operation| operation.coverage.edges.len())
            .sum::<usize>(),
        18
    );
    let mut incomplete = selected.clone();
    incomplete.facts.pop();
    assert!(matches!(
        crate::engine_operation::analyze(&incomplete, &[], limits),
        Err(crate::engine_operation::Error::CaseCensus { .. })
    ));
    assert_eq!(selected.facts.len(), 9);
    assert_eq!(
        selected
            .facts
            .iter()
            .map(|facts| facts.sources.len())
            .sum::<usize>(),
        9
    );
    assert_eq!(
        selected
            .facts
            .iter()
            .map(|facts| facts.runtime.len())
            .sum::<usize>(),
        2
    );
    let mut mismatched = tables.clone();
    mismatched.rows[0]
        .facts
        .as_mut()
        .expect("case facts")
        .case_operation
        .namespace = 1;
    assert_eq!(
        core::select(&mismatched, &proofs[0]).err(),
        Some(core::Error::CaseFactsMismatch(
            mismatched.rows[0].key.clone()
        ))
    );
    let required = selected
        .placement
        .plans
        .iter()
        .map(|(key, _)| key.clone())
        .collect();
    assert!(crate::engine_coverage::validate_census(&required, &selected.coverage).is_ok());
    let mut removed = tables.clone();
    let row = removed.rows.remove(0);
    assert_eq!(
        core::select(&removed, &proofs[0]).err(),
        Some(core::Error::CaseCensusMismatch {
            missing: vec![row.key],
            unexpected: Vec::new()
        })
    );
    let mut duplicate = tables;
    let row = duplicate.rows[0].clone();
    let key = row.key.clone();
    duplicate.rows.push(row);
    assert_eq!(
        core::select(&duplicate, &proofs[0]).err(),
        Some(core::Error::DuplicateCase(key))
    );
}
#[test]
fn attestation_operation_assembly_preserves_factor_census_and_placements() {
    use crate::engine_records::LoweredRelations;
    use std::collections::BTreeSet;
    let input = super::bound_input(&[OperationId::CompactAsh]);
    let registered =
        crate::engine_attestation::cases::lower_registered(&input).expect("case records");
    let tables = &registered.tables;
    let (proofs, _) =
        crate::engine_records::analyze_bound_proofs(&input, &CapabilityView::Unconstrained)
            .expect("proofs");
    let proof = proofs.first().expect("candidate");
    let selected = core::select(tables, proof).expect("selection");
    let relations = input.owner().lower_relations().expect("owner lowering");
    let edges = relations
        .records()
        .edges
        .iter()
        .map(|row| (row.source.clone(), row.target.clone()))
        .collect::<Vec<_>>();
    let limits = crate::placement::PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("states"),
        std::num::NonZeroU64::new(1_000_000).expect("candidates"),
    );
    let operations =
        crate::engine_operation::analyze(&selected, &edges, limits).expect("common assembly");
    let factor = operations.values().next().expect("factor");
    let candidate = crate::engine_attestation::proof::lower(&input)
        .expect("proof registry")
        .candidate(proof.clone());
    let relations = crate::relation::build_relation_analysis(&input).expect("relation analysis");
    let constructibility =
        crate::constructibility::build_constructibility_analysis(&input).expect("constructibility");
    let lifecycle =
        crate::lifecycle::build_lifecycle_analysis(&input, &relations).expect("lifecycle");
    let requirements = crate::requirement::relation_requirements(
        &input,
        &relations,
        &constructibility,
        &lifecycle,
        &candidate,
    )
    .expect("requirements");
    let legacy = crate::analyzed_operation::analyze_operation(
        &relations,
        &requirements,
        &candidate,
        OperationId::CompactAsh,
        limits,
    )
    .expect("compatibility");
    let restored = crate::engine_attestation::operation::restore(
        &registered.payloads,
        &registered.layouts,
        proof,
        factor,
    )
    .expect("lossless factor view");
    assert_eq!(restored, legacy);
    assert_joined_layouts_restore_from_operation_registry(&registered, proof, factor);
    assert_eq!(factor.bundles.len(), legacy.relation_cases.len());
    assert_eq!(factor.cases.len(), legacy.execution_cases.len());
    assert_eq!(
        factor.coverage.nodes.len(),
        legacy.coverage_dependencies.nodes.len()
    );
    assert_eq!(
        factor.coverage.edges.len(),
        legacy.coverage_dependencies.edges.len()
    );
    let placements = crate::analyzed_operation::analyze_operation_placements(
        &relations,
        &candidate,
        OperationId::CompactAsh,
        limits,
    )
    .expect("placement compatibility");
    assert_eq!(
        factor.placements.iter().cloned().collect::<BTreeSet<_>>(),
        placements
            .placements
            .iter()
            .map(crate::engine_attestation::placement::candidate)
            .collect()
    );
    assert_eq!(factor.search, placements.search);
}

fn assert_joined_layouts_restore_from_operation_registry(
    registered: &crate::engine_attestation::cases::Registered,
    proof: &crate::engine_proof::Candidate,
    factor: &crate::engine_operation::Operation,
) {
    let borrowed = factor
        .bundles
        .iter()
        .find_map(|(key, bundle)| {
            let original = registered
                .tables
                .rows
                .iter()
                .find(|row| {
                    row.key == *key && row.proof == proof.proofs.get(&key.relation).cloned()
                })
                .expect("original row");
            let local = &original.facts.as_ref().expect("successful row").layout;
            bundle
                .facts
                .layout
                .iter()
                .find(|layout| !local.contains(*layout))
        })
        .expect("shared join adds a sibling layout");
    let mut missing = registered.layouts.clone();
    missing.remove(borrowed);
    assert_eq!(
        crate::engine_attestation::operation::restore(
            &registered.payloads,
            &missing,
            proof,
            factor
        )
        .err(),
        Some(crate::CompileError::MissingOwnerIdentity)
    );
}
#[test]
fn attestation_tables_match_dynamic_case_discharge_for_each_representation() {
    let input = super::bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    assert_eq!(compare_attestation_eligibility(&input), 4);
    assert_eq!(
        compare_attestation_eligibility(&super::announcement_input()),
        2
    );
}

fn compare_attestation_eligibility(input: &crate::BoundCompilerInput) -> usize {
    let tables = crate::input::AttestationOwner::lower_cases(input).expect("tables before search");
    let (proofs, _) =
        crate::engine_records::analyze_bound_proofs(input, &CapabilityView::Unconstrained)
            .expect("proofs");
    let lowered = crate::engine_attestation::proof::lower(input).expect("compatibility");
    let relations = crate::relation::build_relation_analysis(input).expect("relations");
    let mut seen = std::collections::BTreeSet::new();
    for proof in proofs {
        if !seen.insert(proof.representations.clone()) {
            continue;
        }
        let selected = core::select(&tables, &proof).expect("table selection");
        let candidate = lowered.candidate(proof);
        let cases =
            crate::case::execution_cases(&relations, &candidate).expect("compatibility cases");
        let plans = crate::placement::classify_relation_cases(&relations, &cases)
            .expect("compatibility discharge");
        let eligible = crate::carrier::relation_case_eligibility(&relations, &plans)
            .expect("compatibility eligibility");
        compare_case_facts(&relations, &cases, &plans, &eligible, &selected.facts);
        let expected = crate::engine_attestation::placement::lower(&plans, &eligible, None);
        assert_eq!(
            selected
                .placement
                .plans
                .iter()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            expected.records.plans.into_iter().collect()
        );
        assert_eq!(
            canonical_eligibility(&selected.placement.eligibility),
            canonical_eligibility(&expected.records.eligibility)
        );
        let direct = crate::coverage::analyze_plan_coverage(&relations, &plans)
            .expect("compatibility coverage");
        let rows = direct
            .plans()
            .map(crate::engine_attestation::coverage::lowered)
            .map(|plan| (plan.key.clone(), plan))
            .collect::<std::collections::BTreeMap<_, _>>();
        for row in selected.coverage {
            assert_eq!(row, rows[&row.key]);
        }
    }
    seen.len()
}

fn compare_case_facts(
    relations: &crate::relation::CompilerRelationAnalysis,
    cases: &[crate::case::ExecutionCase],
    plans: &[crate::placement::RelationCasePlan],
    eligibility: &[crate::carrier::CarrierEligibility],
    facts: &[crate::engine_case_facts::Facts],
) {
    for plan in plans {
        let execution = cases
            .iter()
            .find(|case| case.id == plan.case)
            .expect("case");
        let sources = execution
            .active_sources
            .iter()
            .filter(|row| row.operand.relation() == &plan.relation)
            .cloned()
            .collect();
        let eligible = eligibility
            .iter()
            .filter(|row| row.relation == plan.relation && row.case == plan.case)
            .cloned()
            .collect::<Vec<_>>();
        let layout =
            crate::layout::layout_requirements(relations, std::slice::from_ref(plan), &eligible)
                .expect("layout");
        let expected = crate::engine_attestation::case_facts::lower(plan, &sources, &layout);
        let actual = facts
            .iter()
            .find(|facts| facts.key == expected.key)
            .expect("retained case");
        assert_eq!(actual, &expected);
    }
}

// Exhaustive field destructuring makes a new eligibility field a compile-time
// obligation here. Sort vectors, never deduplicate: duplicate rows still matter.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CanonicalEligibility {
    key: crate::engine_placement::Key,
    multiplicity: crate::placement::CarrierMultiplicity,
    entries: Vec<CanonicalEntry>,
    options: Vec<CanonicalEntry>,
    empty_cause: u8,
}
type CanonicalEntry = (
    crate::engine_placement::Placed,
    std::collections::BTreeSet<crate::engine_placement::LayoutKey>,
);
fn canonical_eligibility(
    rows: &[crate::engine_placement::Eligibility],
) -> Vec<CanonicalEligibility> {
    use crate::engine_placement::{Eligibility, EmptyCause, Entry};
    let entries = |values: &[Entry]| {
        let mut result = values
            .iter()
            .map(|Entry { placed, layout }| (placed.clone(), layout.clone()))
            .collect::<Vec<_>>();
        result.sort();
        result
    };
    let mut result = rows
        .iter()
        .map(
            |Eligibility {
                 key,
                 multiplicity,
                 entries: offered,
                 options,
                 empty_cause,
             }| CanonicalEligibility {
                key: key.clone(),
                multiplicity: *multiplicity,
                entries: entries(offered),
                options: entries(options),
                empty_cause: match empty_cause {
                    EmptyCause::NoEligible => 0,
                    EmptyCause::GlobalOnlyLocal => 1,
                    EmptyCause::UnconditionalOptional => 2,
                },
            },
        )
        .collect::<Vec<_>>();
    result.sort();
    result
}

#[test]
fn keyed_complete_eligibility_matches_direct_lowering_in_every_scope() {
    for scope in [
        vec![KeyedOperationId::Source],
        vec![KeyedOperationId::Merge],
        vec![KeyedOperationId::Source, KeyedOperationId::Merge],
    ] {
        let input = keyed_scope(&scope);
        let tables = crate::input::KeyedOwner::lower_cases(&input).expect("case tables");
        let (proofs, _) =
            crate::engine_records::analyze_bound_proofs(&input, &CapabilityView::Unconstrained)
                .expect("proofs");
        assert_ne!(proofs, [] as [crate::engine_proof::Candidate; 0]);
        let expected = crate::engine_keyed::lower_placements(input.owner());
        for proof in proofs {
            let selected = core::select(&tables, &proof).expect("selection");
            assert_eq!(
                canonical_eligibility(&selected.placement.eligibility),
                canonical_eligibility(&expected.eligibility)
            );
            assert_eq!(
                selected.placement.eligibility.len(),
                if scope.contains(&KeyedOperationId::Merge) {
                    2
                } else {
                    0
                }
            );
        }
    }
}

#[test]
fn complete_eligibility_detects_equal_length_coordinate_mutations() {
    use crate::carrier::CarrierQuantification;
    use crate::engine_placement::EmptyCause;
    use crate::placement::CarrierMultiplicity;
    let input = keyed();
    let accepted = crate::engine_keyed::lower_placements(input.owner()).eligibility;
    let expected = canonical_eligibility(&accepted);
    assert_eq!(canonical_eligibility(&accepted), expected);
    for coordinate in 0..8 {
        let mut subject = accepted.clone();
        let row = &mut subject[0];
        match coordinate {
            0 => {
                row.key.relation.operation.namespace = 1;
            }
            1 => {
                row.multiplicity = CarrierMultiplicity::ExactlyOne;
            }
            2 => {
                row.entries[0].placed.quantification = CarrierQuantification::Single;
            }
            3 => {
                row.options[0].placed.quantification = CarrierQuantification::Single;
            }
            4 | 5 => {
                let entry = if coordinate == 4 {
                    &mut row.entries[0]
                } else {
                    &mut row.options[0]
                };
                assert_eq!(
                    entry.placed.carrier,
                    crate::engine_keyed::keyed_carrier(
                        row.key.relation.operation,
                        crate::engine_lifecycle::ObjectKey {
                            namespace: 2,
                            code: 1
                        }
                    )
                );
                entry.placed.carrier = crate::engine_keyed::keyed_carrier(
                    row.key.relation.operation,
                    crate::engine_lifecycle::ObjectKey {
                        namespace: 1,
                        code: 1,
                    },
                );
            }
            6 => {
                row.empty_cause = EmptyCause::GlobalOnlyLocal;
            }
            _ => {
                row.empty_cause = EmptyCause::UnconditionalOptional;
            }
        }
        assert_eq!(subject.len(), accepted.len());
        assert_eq!(subject[0].entries.len(), accepted[0].entries.len());
        assert_eq!(subject[0].options.len(), accepted[0].options.len());
        assert_ne!(
            canonical_eligibility(&subject),
            expected,
            "coordinate {coordinate}"
        );
    }
}

#[test]
fn complete_eligibility_detects_equal_length_layout_mutations() {
    // Layout is empty for keyed plans. Use a real attestation layout obligation
    // as the accepted nonempty comparison subject, then change only its side.
    let attestation = super::bound_input(&[OperationId::CompactAsh]);
    let registered = crate::engine_attestation::cases::lower_registered(&attestation)
        .expect("attestation tables");
    let layouts = registered.layouts.clone();
    let tables = registered.tables;
    let (proofs, _) =
        crate::engine_records::analyze_bound_proofs(&attestation, &CapabilityView::Unconstrained)
            .expect("proofs");
    let selected = core::select(&tables, &proofs[0]).expect("selection");
    let accepted = selected.placement.eligibility;
    let expected = canonical_eligibility(&accepted);
    for option in [false, true] {
        let mut subject = accepted.clone();
        let entry = subject
            .iter_mut()
            .flat_map(|row| {
                if option {
                    &mut row.options
                } else {
                    &mut row.entries
                }
            })
            .find(|entry| {
                entry.layout.iter().any(|key| {
                    matches!(
                        layouts[key],
                        crate::layout::LayoutRequirement::AuthenticateFamilyCensus { .. }
                    )
                })
            })
            .expect("family census layout");
        let original = entry
            .layout
            .iter()
            .find(|key| {
                matches!(
                    layouts[key],
                    crate::layout::LayoutRequirement::AuthenticateFamilyCensus { .. }
                )
            })
            .expect("layout")
            .clone();
        let mut changed = layouts[&original].clone();
        let crate::layout::LayoutRequirement::AuthenticateFamilyCensus { side, .. } = &mut changed
        else {
            panic!("selected layout")
        };
        *side = match side {
            realization::TransactionSide::Input => realization::TransactionSide::Output,
            realization::TransactionSide::Output => realization::TransactionSide::Input,
        };
        let count = entry.layout.len();
        assert!(entry.layout.remove(&original));
        assert!(
            entry
                .layout
                .insert(crate::engine_attestation::identity::attestation_layout(
                    &changed
                ))
        );
        assert_eq!(entry.layout.len(), count);
        assert_eq!(subject.len(), accepted.len());
        assert_ne!(canonical_eligibility(&subject), expected);
    }
    let mut permuted = accepted.clone();
    permuted.reverse();
    for row in &mut permuted {
        row.entries.reverse();
        row.options.reverse();
    }
    assert_eq!(canonical_eligibility(&permuted), expected);
    let mut duplicate = accepted;
    duplicate.push(duplicate[0].clone());
    assert_ne!(canonical_eligibility(&duplicate), expected);
}
