//! The shared validator owns runtime assignment refusal for both vocabularies.
use crate::{engine_placement as core, engine_records::RelationOwner};
use architecture::{
    OperationId,
    keyed::KeyedManifest,
    semantic::{KeyedOperationId, SemanticOperationId},
};

fn bound(scope: Vec<KeyedOperationId>) -> crate::BoundCompilerInput<crate::input::KeyedOwner> {
    let manifest = KeyedManifest::contract().expect("manifest");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("declaration")
        .validate(&manifest)
        .expect("validation");
    crate::bind_input(
        &manifest,
        source,
        scope.into_iter().map(SemanticOperationId::Keyed).collect(),
        super::bound_input(&[OperationId::CompactAsh]).policy(),
    )
    .expect("binding")
}
fn limits() -> crate::placement::PlacementSearchLimits {
    crate::placement::PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1000).expect("states"),
        std::num::NonZeroU64::new(100).expect("candidates"),
    )
}
fn records(scope: Vec<KeyedOperationId>) -> core::Records {
    let input = bound(scope);
    crate::engine_keyed::lower_placements(input.owner())
}
fn candidates(records: &core::Records) -> Result<Vec<core::Candidate>, core::Error> {
    core::enumerate(
        records,
        limits(),
        &mut crate::placement::PlacementSearchReport::default(),
    )
}

#[test]
fn shared_placement_tracing_reports_every_return_without_changing_results() {
    use super::tracing_capture::{FieldValue, capture, isolate_capture_test};
    if isolate_capture_test(
        "tests::engine_placement_tests::shared_placement_tracing_reports_every_return_without_changing_results",
    ) {
        return;
    }
    let control = records(vec![KeyedOperationId::Merge]);
    let mut census = control.clone();
    census.eligibility.pop();
    let mut unavailable = control.clone();
    unavailable.eligibility[0].options.clear();
    unavailable.eligibility[0].empty_cause = core::EmptyCause::NoEligible;
    let mut unpermitted = control.clone();
    unpermitted.eligibility[0].entries.clear();
    let mut multiple = control.clone();
    let extra = multiple.eligibility[0].options[0].clone();
    multiple.eligibility[0].options.push(extra);
    let one = std::num::NonZeroU64::new(1).unwrap();
    for (records, limits, refusal) in [
        (&control, limits(), None),
        (&census, limits(), Some("placement_census_mismatch")),
        (&unavailable, limits(), Some("no_eligible_carrier")),
        (
            &unpermitted,
            limits(),
            Some("unpermitted_carrier_placement"),
        ),
        (
            &control,
            crate::placement::PlacementSearchLimits::new(one, limits().maximum_candidates),
            Some("placement_search_state_limit_exceeded"),
        ),
        (
            &multiple,
            crate::placement::PlacementSearchLimits::new(limits().maximum_states, one),
            Some("placement_candidate_limit_exceeded"),
        ),
    ] {
        let mut plain_report = crate::placement::PlacementSearchReport::default();
        let plain = core::enumerate(records, limits, &mut plain_report);
        assert_eq!(plain.is_err(), refusal.is_some());
        let mut report = crate::placement::PlacementSearchReport::default();
        let (observed, events) = capture(|| core::enumerate(records, limits, &mut report));
        assert_eq!(observed, plain);
        assert_eq!(report, plain_report);
        let events = events
            .iter()
            .filter(|event| event.target == "compiler::engine_placement")
            .collect::<Vec<_>>();
        assert_eq!(
            events.len(),
            2,
            "one span and one completion, no candidate spans"
        );
        assert!(events[0].is_span);
        assert_eq!(
            events[0].fields.get("state_limit"),
            Some(&FieldValue::U64(limits.maximum_states.get()))
        );
        assert_eq!(
            events[0].fields.get("candidate_limit"),
            Some(&FieldValue::U64(limits.maximum_candidates.get()))
        );
        let event = events[1];
        assert!(!event.is_span);
        assert_eq!(
            event.fields.get("message"),
            Some(&FieldValue::Str("placement search completed".into()))
        );
        assert_eq!(
            event.fields.get("outcome"),
            Some(&FieldValue::Str(
                if refusal.is_some() {
                    "refused"
                } else {
                    "completed"
                }
                .into()
            ))
        );
        assert_eq!(
            event.fields.get("refusal_kind"),
            refusal.map(|kind| FieldValue::Str(kind.into())).as_ref()
        );
        for (field, value) in [
            ("visited_state_count", report.states_visited),
            ("complete_assignment_count", report.complete_assignments),
            ("feasible_placement_count", report.feasible_placements),
            ("retained_option_count", report.retained_options),
        ] {
            assert_eq!(event.fields.get(field), Some(&FieldValue::U64(value)));
        }
    }
}

#[test]
fn evidence_only_source_has_no_runtime_carrier_and_merge_has_two_assignments() {
    for (scope, plans, runtime) in [
        (vec![KeyedOperationId::Source], 4, 0),
        (vec![KeyedOperationId::Merge], 5, 2),
        (
            vec![KeyedOperationId::Source, KeyedOperationId::Merge],
            9,
            2,
        ),
    ] {
        let records = records(scope);
        assert_eq!(records.plans.len(), plans);
        assert_eq!(records.eligibility.len(), runtime);
        let mut report = crate::placement::PlacementSearchReport::default();
        let placements = core::enumerate(&records, limits(), &mut report).expect("placement");
        assert_eq!(placements.len(), 1);
        assert_eq!(placements[0].assignments.len(), runtime);
        assert_eq!(placements[0].layout, Vec::<core::LayoutKey>::new());
        assert_eq!(
            report.states_visited,
            u64::try_from(runtime).expect("census") + 1
        );
        assert!(core::validate_placement(&records, &placements[0]).is_ok());
    }
}

#[test]
fn shared_validator_refuses_reader_runtime_assignment() {
    let records = records(vec![KeyedOperationId::Source, KeyedOperationId::Merge]);
    let accepted = candidates(&records).expect("accepted").remove(0);
    assert!(core::validate_placement(&records, &accepted).is_ok());
    let input = bound(vec![KeyedOperationId::Source, KeyedOperationId::Merge]);
    let sources = input.owner().lower_sources().expect("sources");
    let reader = sources
        .tables
        .iter()
        .find(|table| {
            table.rows.iter().any(|row| {
                matches!(
                    row.operand.payload,
                    crate::engine_source::Payload::Evidence(
                        crate::engine_source::Evidence::MembershipReading { .. }
                    )
                )
            })
        })
        .expect("reader");
    let mut subject = accepted;
    let mut runtime = subject.assignments[0].clone();
    runtime.key = core::Key {
        relation: reader.relation.clone(),
        case: core::CaseKey::Keyed(reader.relation.operation),
    };
    let unexpected = runtime.key.clone();
    subject.assignments.push(runtime);
    assert_eq!(
        core::validate_placement(&records, &subject),
        Err(core::Error::PlacementCensusMismatch {
            missing: Vec::new(),
            unexpected: vec![unexpected]
        })
    );
}

#[test]
fn missing_duplicate_unreachable_and_mismatched_assignments_are_refused() {
    let records = records(vec![KeyedOperationId::Merge]);
    let accepted = candidates(&records).expect("accepted").remove(0);
    assert!(core::validate_placement(&records, &accepted).is_ok());
    let mut missing = accepted.clone();
    let removed = missing.assignments.remove(0);
    assert_eq!(
        core::validate_placement(&records, &missing),
        Err(core::Error::PlacementCensusMismatch {
            missing: vec![removed.key],
            unexpected: Vec::new()
        })
    );
    let mut duplicate = accepted.clone();
    duplicate.assignments.push(duplicate.assignments[0].clone());
    assert_eq!(
        core::validate_placement(&records, &duplicate),
        Err(core::Error::PlacementCensusMismatch {
            missing: Vec::new(),
            unexpected: Vec::new()
        })
    );
    let mut mismatched = accepted.clone();
    mismatched.assignments[0].carriers[0].quantification =
        crate::carrier::CarrierQuantification::Single;
    assert!(matches!(
        core::validate_placement(&records, &mismatched),
        Err(core::Error::UnpermittedCarrierPlacement { .. })
    ));
    let mut unavailable_options = records.clone();
    unavailable_options.eligibility[0].options.clear();
    assert_eq!(
        candidates(&unavailable_options).err(),
        Some(core::Error::NoEligibleCarrier(
            unavailable_options.eligibility[0].key.clone()
        ))
    );
    let mut unreachable = records.clone();
    unreachable.eligibility[0].entries.clear();
    assert!(matches!(
        core::validate_placement(&unreachable, &accepted),
        Err(core::Error::UnpermittedCarrierPlacement { .. })
    ));
    let mut carrier_duplicate = accepted;
    let repeated = carrier_duplicate.assignments[0].carriers[0].clone();
    carrier_duplicate.assignments[0].carriers.push(repeated);
    assert!(matches!(
        core::validate_placement(&records, &carrier_duplicate),
        Err(core::Error::DuplicatePlacedCarrier { .. })
    ));
}
