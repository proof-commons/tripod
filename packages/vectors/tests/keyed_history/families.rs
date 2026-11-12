//! Family controls compare byte claims with graph-only expectations and literals.
use super::{fixture, model, reader, reading};
use std::collections::BTreeSet;
use std::io::Write;
use vectors::{keyed_evidence as e, keyed_history_reader as r};
const FAMILIES: [e::Family; 6] = [
    e::Family::Birth,
    e::Family::Continuation,
    e::Family::Recurrence,
    e::Family::Reading,
    e::Family::Acceptance,
    e::Family::Resources,
];
type Validator = fn(&e::FamilyContext, &e::Request, &e::Report) -> Result<(), e::FamilyRefusal>;
pub(super) fn validator(f: e::Family) -> Validator {
    match f {
        e::Family::Birth => e::validate_birth,
        e::Family::Continuation => e::validate_continuation,
        e::Family::Recurrence => e::validate_recurrence,
        e::Family::Reading => e::validate_reading,
        e::Family::Acceptance => e::validate_acceptance,
        e::Family::Resources => e::validate_resources,
    }
}
const fn handle(h: model::Handle) -> r::Handle {
    r::Handle {
        key: r::Key(h.key),
        output: r::Output {
            event: h.output.event,
            position: h.output.position,
        },
    }
}
pub(super) fn subject(name: &str) {
    let fixture = match name {
        "cut-two" => fixture::existing(2),
        "cut-253" => fixture::existing(253),
        _ => fixture::build(fixture::graph(name)),
    };
    let input = reader::input(fixture.reader_inputs());
    let context = e::prepare_families(&input).unwrap();
    writeln!(
        std::io::stdout().lock(),
        "family index subject={name} second-pass-wall={:.9}s",
        context.second_pass_wall().as_secs_f64()
    )
    .unwrap();
    let event = *input.bytes.keys().last().unwrap();
    let mut request = reading::request(&input, event);
    request.handle.key = r::Key(
        fixture
            .graph
            .iter()
            .find(|row| row.id == event)
            .unwrap()
            .outputs[0]
            .unwrap(),
    );
    request.completeness = e::Completeness::Report;
    for family in FAMILIES {
        let report = e::family_report(&context, family, &request).unwrap();
        validator(family)(&context, &request, &report).unwrap();
        expected(&fixture, event, &report, name.starts_with("cut-"));
        common(&context, family, &request, &report);
        specific(&context, family, &request, &report);
    }
    if name == "multi" {
        super::family_controls::rules(&context, &request);
        missing(&input, &request);
    }
}
fn cut_dependencies(fixture: &fixture::Fixture) -> BTreeSet<r::Missing> {
    fixture
        .boundary_premises
        .iter()
        .flat_map(|event| {
            fixture.external_inputs[event]
                .iter()
                .enumerate()
                .map(|(input, (address, position))| r::Missing::Parent {
                    event: *event,
                    input,
                    address: r::Address(*address),
                    position: *position,
                })
        })
        .collect()
}
fn expected(fixture: &fixture::Fixture, event: u32, report: &e::Report, cut: bool) {
    let expected = fixture
        .expected
        .records(model::Handle {
            key: fixture
                .graph
                .iter()
                .find(|row| row.id == event)
                .unwrap()
                .outputs[0]
                .unwrap(),
            output: model::Output { event, position: 0 },
        })
        .unwrap()
        .iter()
        .copied()
        .map(handle)
        .collect::<BTreeSet<_>>();
    identities(fixture, event, report);
    match &report.body {
        e::Body::Birth(body) => {
            assert_eq!(
                body.first_outputs,
                BTreeSet::from([r::Handle {
                    key: r::Key(
                        fixture
                            .graph
                            .iter()
                            .find(|row| row.id == event)
                            .unwrap()
                            .outputs[0]
                            .unwrap()
                    ),
                    output: r::Output { event, position: 0 }
                }])
            );
            assert!(
                body.relations
                    .iter()
                    .all(|row| row.relation == r::Relation::BornFirst && row.member.position == 0)
            );
        }
        e::Body::Continuation(body) => assert!(
            body.relations
                .iter()
                .all(|row| row.relation == r::Relation::ContinueFirst && row.member.position == 0)
        ),
        e::Body::Recurrence(body) => match &body.outcome {
            e::Outcome::Complete(row) => {
                assert!(!cut);
                assert_eq!(row.records, expected);
                recurrence_expected(fixture, event, row);
            }
            e::Outcome::Incomplete { missing, .. } => {
                assert!(cut);
                assert_eq!(*missing, cut_dependencies(fixture));
            }
            other => panic!("{other:?}"),
        },
        e::Body::Reading(body) => match &body.outcome {
            e::Outcome::Complete(row) => {
                assert!(!cut);
                assert_eq!(row.membership, expected);
            }
            e::Outcome::Incomplete { missing, .. } => {
                assert!(cut);
                assert_eq!(*missing, cut_dependencies(fixture));
            }
            other => panic!("{other:?}"),
        },
        e::Body::Acceptance(body) => match &body.reading.outcome {
            e::Outcome::Complete(row) => {
                assert!(!cut);
                assert_eq!(row.reading.membership, expected);
                assert_eq!(row.accepted, expected);
            }
            e::Outcome::Incomplete { missing, .. } => {
                assert!(cut);
                assert_eq!(*missing, cut_dependencies(fixture));
            }
            other => panic!("{other:?}"),
        },
        e::Body::Resources(body) => resources(fixture, event, body),
        _ => panic!("new family needs independent expectation"),
    }
}
fn recurrence_expected(fixture: &fixture::Fixture, event: u32, row: &e::Recurrence) {
    let declared = fixture.graph.iter().find(|row| row.id == event).unwrap();
    let handles: BTreeSet<_> = declared
        .inputs
        .iter()
        .filter_map(|point| {
            let parent = fixture
                .graph
                .iter()
                .find(|row| row.id == point.event)
                .unwrap();
            let key = parent.outputs[point.position]?;
            (point.position == 0).then_some(model::Handle {
                key,
                output: *point,
            })
        })
        .collect();
    assert_eq!(
        row.source,
        handles.is_empty().then_some(r::Handle {
            key: r::Key(declared.outputs[0].unwrap()),
            output: r::Output { event, position: 0 }
        })
    );
    assert_eq!(
        row.consumed.keys().copied().collect::<BTreeSet<_>>(),
        handles.iter().copied().map(handle).collect()
    );
    for parent in handles {
        assert_eq!(
            row.consumed[&handle(parent)],
            r::Membership::Complete(
                fixture
                    .expected
                    .records(parent)
                    .unwrap()
                    .iter()
                    .copied()
                    .map(handle)
                    .collect()
            )
        );
    }
}

pub(super) fn refuse(
    c: &e::FamilyContext,
    f: e::Family,
    q: &e::Request,
    report: &e::Report,
    condition: e::FamilyCondition,
) {
    let error = validator(f)(c, q, report).unwrap_err();
    let boundary = error.boundary();
    let observation = error.observation_boundary();
    assert_eq!(
        (error.condition, error.family, boundary, observation),
        (
            condition,
            Some(f),
            e::FamilyBoundary::FamilyReportValidation,
            None
        )
    );
}
fn common(c: &e::FamilyContext, f: e::Family, q: &e::Request, good: &e::Report) {
    for choice in 0..6 {
        let mut bad = good.clone();
        let error = match choice {
            0 => {
                bad.header.subject.as_mut().unwrap().bytes[0] ^= 1;
                e::Refusal::SubjectBytesMismatch
            }
            1 => {
                bad.header.family = if f == e::Family::Birth {
                    e::Family::Reading
                } else {
                    e::Family::Birth
                };
                e::Refusal::FamilyMismatch
            }
            2 => {
                bad.header.subject.as_mut().unwrap().handle.key = r::Key([99; 32]);
                e::Refusal::SubjectHandleMismatch
            }
            3 => {
                bad.header.assumptions.as_mut().unwrap().pop();
                e::Refusal::AssumptionsMismatch
            }
            4 => {
                bad.header.subject = None;
                e::Refusal::MissingField(e::Field::Subject)
            }
            _ => {
                bad.header.evidence.as_mut().unwrap().keys.clear();
                e::Refusal::EvidenceMismatch
            }
        };
        refuse(c, f, q, &bad, e::FamilyCondition::Framing(error));
    }
    for other in FAMILIES {
        if other != f {
            refuse(
                c,
                other,
                q,
                good,
                e::FamilyCondition::Framing(e::Refusal::FamilyMismatch),
            );
            // Correct validator family in the header, but the wrong body.
            let mut bad = good.clone();
            bad.body = e::family_report(c, other, q).unwrap().body;
            refuse(c, f, q, &bad, e::FamilyCondition::Body);
        }
    }
}
fn specific(c: &e::FamilyContext, f: e::Family, q: &e::Request, good: &e::Report) {
    let mut bad = good.clone();
    let condition = match &mut bad.body {
        e::Body::Birth(body) => {
            body.first_outputs = BTreeSet::from([r::Handle {
                output: r::Output {
                    event: q.handle.output.event,
                    position: 1,
                },
                ..q.handle
            }]);
            e::FamilyCondition::FirstOutputs
        }
        e::Body::Continuation(body) => {
            body.successors.clear();
            if body.relations.is_empty() {
                body.trace.requirements.clear();
                e::FamilyCondition::Trace
            } else {
                e::FamilyCondition::Successors
            }
        }
        e::Body::Recurrence(body) => match &mut body.outcome {
            e::Outcome::Complete(row) => {
                row.records.insert(if row.source.is_none() {
                    q.handle
                } else {
                    reading::handle(900)
                });
                e::FamilyCondition::RecurrenceRecords
            }
            e::Outcome::Incomplete { missing, .. } => {
                missing.clear();
                e::FamilyCondition::Framing(e::Refusal::OutcomeMismatch)
            }
            _ => panic!(),
        },
        e::Body::Reading(body) => match &mut body.outcome {
            e::Outcome::Complete(row) => {
                row.membership.insert(reading::handle(900));
                e::FamilyCondition::Framing(e::Refusal::MembershipMismatch)
            }
            e::Outcome::Incomplete { missing, .. } => {
                missing.clear();
                e::FamilyCondition::Framing(e::Refusal::OutcomeMismatch)
            }
            _ => panic!(),
        },
        e::Body::Acceptance(body) => match &mut body.reading.outcome {
            e::Outcome::Complete(row) => {
                row.reading.membership.insert(reading::handle(900));
                e::FamilyCondition::Framing(e::Refusal::MembershipMismatch)
            }
            e::Outcome::Incomplete { missing, .. } => {
                missing.clear();
                e::FamilyCondition::Framing(e::Refusal::OutcomeMismatch)
            }
            _ => panic!(),
        },
        e::Body::Resources(body) => {
            body.transaction_weight.predicted = e::Measurement::Present {
                value: 1,
                unit: e::Unit::WeightUnits,
            };
            e::FamilyCondition::ResourceCount
        }
        _ => panic!(),
    };
    refuse(c, f, q, &bad, condition);
    relation_mutants(c, f, q, good);
    more_mutants(c, f, q, good);
    special_requests(c, f, q, good);
}
fn special_requests(c: &e::FamilyContext, f: e::Family, q: &e::Request, good: &e::Report) {
    if matches!(
        f,
        e::Family::Birth | e::Family::Continuation | e::Family::Resources
    ) {
        let mut nonhandle = q.clone();
        nonhandle.handle.output.position = 1;
        let error = e::family_report(c, f, &nonhandle).unwrap_err();
        assert_eq!(
            error.condition,
            e::FamilyCondition::Observation(e::Refusal::Query(r::Query::NotHandle))
        );
        assert_eq!(error.boundary(), e::FamilyBoundary::FamilyReportValidation);
        assert_eq!(
            error.observation_boundary(),
            Some(e::FamilyObservationBoundary::Evidence(
                e::Boundary::ReaderEvidence
            ))
        );
    }

    let mut source_mutant = good.clone();
    if let e::Body::Recurrence(body) = &mut source_mutant.body
        && let e::Outcome::Complete(row) = &mut body.outcome
    {
        row.source = Some(reading::handle(900));
        refuse(
            c,
            f,
            q,
            &source_mutant,
            e::FamilyCondition::RecurrenceSource,
        );
    }

    let mut bad_unit = good.clone();
    if let e::Body::Resources(body) = &mut bad_unit.body {
        let e::Measurement::Present { unit, .. } = &mut body.transaction_weight.predicted else {
            panic!("weight prediction is present")
        };
        *unit = e::Unit::Bytes;
        refuse(c, f, q, &bad_unit, e::FamilyCondition::ResourceUnit);
    }
}
fn relation_mutants(c: &e::FamilyContext, f: e::Family, q: &e::Request, good: &e::Report) {
    for choice in 0..8 {
        let mut bad = good.clone();
        let rows = match &mut bad.body {
            e::Body::Birth(b) => &mut b.relations,
            e::Body::Continuation(b) => &mut b.relations,
            _ => return,
        };
        if rows.is_empty() {
            return;
        }
        let unchanged = rows.get(1).cloned();
        let expected = match choice {
            0 => {
                rows[0].event += 100;
                e::FamilyCondition::RelationEvent
            }
            1 => {
                rows[0].input += 1;
                e::FamilyCondition::RelationInput
            }
            2 => {
                rows[0].member.key = r::Key([99; 32]);
                e::FamilyCondition::RelationMember
            }
            3 => {
                rows[0].relation = if rows[0].relation == r::Relation::BornFirst {
                    r::Relation::ContinueFirst
                } else {
                    r::Relation::BornFirst
                };
                e::FamilyCondition::RelationKind
            }
            4 => {
                rows[0].leaf[0] ^= 1;
                e::FamilyCondition::RelationLeaf
            }
            5 => {
                rows[0].control[0] ^= 1;
                e::FamilyCondition::RelationControl
            }
            6 => {
                rows[0].primitive_count += 1;
                e::FamilyCondition::RelationCounters
            }
            _ => {
                rows.pop();
                e::FamilyCondition::Relations
            }
        };
        if choice != 7
            && let Some(unchanged) = unchanged
        {
            assert_eq!(rows[1], unchanged);
        }
        refuse(c, f, q, &bad, expected);
    }
}

fn resources(fixture: &fixture::Fixture, event: u32, body: &e::ResourcesBody) {
    let bytes = &fixture.bytes[&event];
    let tx: elements::Transaction = elements::encode::deserialize(bytes).unwrap();
    assert_eq!(
        body.transaction_weight.direct,
        e::Measurement::Present {
            value: tx.weight() as u64,
            unit: e::Unit::WeightUnits
        }
    );
    for row in &body.schedules {
        let (length, ops) = match row.schedule {
            e::Schedule::Birth => (10, 5),
            e::Schedule::Continuation => (7, 6),
            e::Schedule::Leaf => (52, 12),
        };
        assert_eq!(row.script.len(), length);
        assert_eq!(
            row.operations.model,
            e::Measurement::Present {
                value: ops,
                unit: e::Unit::Operations
            }
        );
    }
    assert_eq!(
        body.script_stack_bytes.direct,
        e::Measurement::Present {
            value: 88 * body.members,
            unit: e::Unit::Bytes
        }
    );
    assert_eq!(
        body.member_witness_bytes.direct,
        e::Measurement::Present {
            value: 91 * body.members,
            unit: e::Unit::Bytes
        }
    );
    assert_eq!(
        body.member_weight.direct,
        e::Measurement::Present {
            value: 255 * body.members,
            unit: e::Unit::WeightUnits
        }
    );
}
fn missing(input: &r::Input, request: &e::Request) {
    for inclusion in [false, true] {
        let mut missing = input.clone();
        let address = input
            .association
            .iter()
            .find(|(id, _)| *id == 10)
            .unwrap()
            .1;
        let item = if inclusion {
            missing.assertions.inclusion.remove(&address);
            r::Missing::Inclusion(address)
        } else {
            missing.bytes.remove(&10);
            r::Missing::Parent {
                event: 20,
                input: 0,
                address,
                position: 0,
            }
        };
        let c = e::prepare_families(&missing).unwrap();
        for f in [
            e::Family::Recurrence,
            e::Family::Reading,
            e::Family::Acceptance,
        ] {
            let report = e::family_report(&c, f, request).unwrap();
            validator(f)(&c, request, &report).unwrap();
            let missing = match &report.body {
                e::Body::Recurrence(b) => match &b.outcome {
                    e::Outcome::Incomplete { missing, .. } => missing,
                    _ => panic!(),
                },
                e::Body::Reading(b) => match &b.outcome {
                    e::Outcome::Incomplete { missing, .. } => missing,
                    _ => panic!(),
                },
                e::Body::Acceptance(b) => match &b.reading.outcome {
                    e::Outcome::Incomplete { missing, .. } => missing,
                    _ => panic!(),
                },
                _ => panic!(),
            };
            assert_eq!(*missing, BTreeSet::from([item.clone()]));
        }
    }
}

fn identities(fixture: &fixture::Fixture, event: u32, report: &e::Report) {
    let declared = fixture.graph.iter().find(|row| row.id == event).unwrap();
    let expected: Vec<_> = declared
        .inputs
        .iter()
        .enumerate()
        .filter_map(|(index, point)| {
            let parent = fixture
                .graph
                .iter()
                .find(|row| row.id == point.event)
                .unwrap();
            let key = parent.outputs[point.position]?;
            if point.position != 0 {
                return None;
            }
            Some((
                index,
                r::Member {
                    key: r::Key(key),
                    address: r::Address(
                        fixture
                            .association
                            .iter()
                            .find(|(id, _)| *id == point.event)
                            .unwrap()
                            .1,
                    ),
                    position: 0,
                },
            ))
        })
        .collect();
    if let e::Body::Resources(body) = &report.body {
        assert_eq!(body.members, expected.len() as u64);
        assert_eq!(body.schedules.len(), 3 * expected.len());
    }
    if let e::Body::Continuation(body) = &report.body {
        let successor = r::Handle {
            key: r::Key(declared.outputs[0].unwrap()),
            output: r::Output { event, position: 0 },
        };
        let edges = declared
            .inputs
            .iter()
            .filter_map(|point| {
                let parent = fixture
                    .graph
                    .iter()
                    .find(|row| row.id == point.event)
                    .unwrap();
                let key = parent.outputs[point.position]?;
                (point.position == 0).then_some((
                    handle(model::Handle {
                        key,
                        output: *point,
                    }),
                    successor,
                ))
            })
            .collect();
        assert_eq!(body.successors, edges);
    }
    let rows = match &report.body {
        e::Body::Birth(body) => &body.relations,
        e::Body::Continuation(body) => &body.relations,
        _ => return,
    };
    assert_eq!(
        rows.iter()
            .map(|row| (row.input, row.member))
            .collect::<Vec<_>>(),
        expected
    );
    for row in rows {
        assert_eq!(
            (row.peak, row.maximum_width),
            if row.relation == r::Relation::BornFirst {
                (3, 32)
            } else {
                (4, 32)
            }
        );
        assert_eq!(
            &row.leaf[34..44],
            &[0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75]
        );
        assert_eq!(&row.leaf[44..51], &[0xcd, 0xca, 0, 0xd1, 0x7b, 0x88, 0x88]);
        assert_eq!(row.leaf.len(), 52);
        assert_eq!(row.control.len(), 33);
    }
}

fn more_mutants(c: &e::FamilyContext, f: e::Family, q: &e::Request, good: &e::Report) {
    let mut bad = good.clone();
    if let e::Body::Recurrence(b) = &mut bad.body
        && let e::Outcome::Complete(row) = &mut b.outcome
    {
        row.consumed.insert(
            reading::handle(999),
            r::Membership::Complete(BTreeSet::new()),
        );
        refuse(c, f, q, &bad, e::FamilyCondition::RecurrenceInputs);
    }
    let mut bad = good.clone();
    let changed = match &mut bad.body {
        e::Body::Reading(b) => {
            b.outcome = e::Outcome::NotHandle(q.handle);
            true
        }
        e::Body::Recurrence(b) => {
            b.outcome = e::Outcome::NotHandle(q.handle);
            true
        }
        e::Body::Acceptance(b) => {
            b.reading.outcome = e::Outcome::NotHandle(q.handle);
            true
        }
        _ => false,
    };
    if changed {
        refuse(
            c,
            f,
            q,
            &bad,
            e::FamilyCondition::Framing(e::Refusal::OutcomeMismatch),
        );
    }
    if matches!(
        f,
        e::Family::Birth | e::Family::Continuation | e::Family::Resources
    ) {
        let mut final_request = q.clone();
        final_request.finality = e::FinalityRequest::Required;
        let error = e::family_report(c, f, &final_request).unwrap_err();
        assert_eq!(error.condition, e::FamilyCondition::FinalityUnsupported);
        assert_eq!(error.observation_boundary(), None);
        assert_eq!(error.family, Some(f));
        assert_eq!(error.boundary(), e::FamilyBoundary::FamilyReportValidation);
    }
    if f == e::Family::Resources {
        resource_mutants(c, q, good);
    }
}
fn resource_mutants(c: &e::FamilyContext, q: &e::Request, good: &e::Report) {
    let f = e::Family::Resources;
    for choice in 0..8 {
        let mut bad = good.clone();
        let e::Body::Resources(b) = &mut bad.body else {
            panic!()
        };
        let error = match choice {
            0 => {
                b.base_bytes += 1;
                e::FamilyCondition::ResourceTerms
            }
            1 => {
                b.transaction_weight.direct = e::Measurement::Absent(e::Absence::NotObservedHere);
                e::FamilyCondition::ResourceAvailability
            }
            _ if b.schedules.is_empty() => continue,
            2 => {
                b.schedules[0].control[0] ^= 1;
                e::FamilyCondition::Control
            }
            3 => {
                b.schedules[0].script[0] ^= 1;
                e::FamilyCondition::Script
            }
            4 => {
                b.schedules[0].execution_arguments += 1;
                e::FamilyCondition::ExecutionArguments
            }
            5 => {
                b.schedules[0].dimensions.clear();
                e::FamilyCondition::ProjectionTerms
            }
            6 => {
                b.schedules.pop();
                e::FamilyCondition::ScheduleIdentity
            }
            _ => {
                b.schedules[0].script_bytes.model = e::Measurement::Present {
                    value: 1,
                    unit: e::Unit::Bytes,
                };
                e::FamilyCondition::ResourceAvailability
            }
        };
        refuse(c, f, q, &bad, error);
    }
    let mut bad = good.clone();
    bad.header
        .quantities
        .get_mut(&e::Dimension::Weight)
        .unwrap()
        .observed = e::Measurement::Present {
        value: 1,
        unit: e::Unit::WeightUnits,
    };
    refuse(
        c,
        f,
        q,
        &bad,
        e::FamilyCondition::Framing(e::Refusal::QuantityValue(e::Dimension::Weight)),
    );
    resource_literals(good);
}
fn resource_literals(good: &e::Report) {
    let e::Body::Resources(b) = &good.body else {
        panic!()
    };
    assert_eq!(
        good.header.quantities[&e::Dimension::Weight].observed,
        b.transaction_weight.direct
    );
    assert_eq!(
        good.header.quantities[&e::Dimension::Weight].predicted,
        b.transaction_weight.predicted
    );
    assert_eq!(
        good.header.quantities[&e::Dimension::Records].observed,
        e::Measurement::Absent(e::Absence::NotObservedHere)
    );
    for s in &b.schedules {
        let peak = if s.schedule == e::Schedule::Birth {
            3
        } else {
            4
        };
        assert_eq!(
            s.peak.model,
            e::Measurement::Present {
                value: peak,
                unit: e::Unit::Items
            }
        );
        assert_eq!(
            s.element_width.model,
            e::Measurement::Present {
                value: 32,
                unit: e::Unit::Bytes
            }
        );
        assert_eq!(
            s.element_width.predicted,
            e::Measurement::Present {
                value: if s.schedule == e::Schedule::Birth {
                    32
                } else {
                    40
                },
                unit: e::Unit::Bound
            }
        );
        assert_eq!(s.execution_arguments, 0);
        assert_eq!(s.execution_arguments_unit, e::Unit::Items);
    }
}
