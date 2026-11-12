//! One-coordinate report corruptions beside a freshly validated report.
use super::{fixture, reader, reading};
use vectors::keyed_evidence as e;

pub(super) fn controls() {
    let fixture = fixture::build(fixture::graph("multi"));
    let input = reader::input(fixture.reader_inputs());
    let request = reading::request(&input, 20);
    let accepted = e::reading_report(&input, &request).unwrap();
    assert_eq!(e::SCHEMA, 1);
    assert_eq!(accepted.header.schema, 1);
    trace(&accepted.header.trace.clone().unwrap());
    trace(&reading::body(&accepted).reading.trace);
    assert_eq!(accepted.header.family, e::Family::Acceptance);
    let mut query = request.clone();
    query.family = e::ReadingFamily::Reading;
    let report = e::reading_report(&input, &query).unwrap();
    let e::Body::Reading(body) = &report.body else {
        panic!("reading body")
    };
    trace(&body.trace);
    assert_eq!(e::validate_report(&input, &query, &report), Ok(()));
    let mut foreign = accepted.clone();
    foreign.body = report.body.clone();
    assert_eq!(
        e::validate_report(&input, &request, &foreign),
        Err(e::Refusal::BodyMismatch)
    );
    let mut wrong_trace = accepted.clone();
    reading::body_mut(&mut wrong_trace)
        .reading
        .trace
        .paper
        .clear();
    assert_eq!(
        e::validate_report(&input, &request, &wrong_trace),
        Err(e::Refusal::TraceMismatch)
    );
    missing(&input, &request, &accepted);
    contradictions(&input, &request, &accepted);
    quantities(&input, &request, &accepted);
    provenance(&input, &request);
    let mut different = input.clone();
    different.bytes.get_mut(&10).unwrap()[0] ^= 1;
    assert_eq!(e::validate_report(&input, &request, &accepted), Ok(()));
    assert!(matches!(
        e::validate_report(&different, &request, &accepted),
        Err(e::Refusal::Reader(_))
    ));
}
fn missing(
    input: &vectors::keyed_history_reader::Input,
    request: &e::Request,
    accepted: &e::Report,
) {
    for field in [
        e::Field::Subject,
        e::Field::Provenance,
        e::Field::Trace,
        e::Field::Evidence,
        e::Field::Assumptions,
    ] {
        let mut subject = accepted.clone();
        match field {
            e::Field::Subject => subject.header.subject = None,
            e::Field::Provenance => subject.header.provenance = None,
            e::Field::Trace => subject.header.trace = None,
            e::Field::Evidence => subject.header.evidence = None,
            e::Field::Assumptions => subject.header.assumptions = None,
        }
        assert_eq!(e::validate_report(input, request, accepted), Ok(()));
        assert_eq!(
            e::validate_report(input, request, &subject),
            Err(e::Refusal::MissingField(field))
        );
    }
}
fn contradictions(
    input: &vectors::keyed_history_reader::Input,
    request: &e::Request,
    accepted: &e::Report,
) {
    let errors = [
        e::Refusal::SchemaMismatch,
        e::Refusal::FamilyMismatch,
        e::Refusal::SubjectBytesMismatch,
        e::Refusal::SubjectIdentityMismatch,
        e::Refusal::SubjectHandleMismatch,
        e::Refusal::ProvenanceMismatch,
        e::Refusal::TraceMismatch,
        e::Refusal::RuleMismatch,
        e::Refusal::EvidenceMismatch,
        e::Refusal::AssumptionsMismatch,
        e::Refusal::OutcomeMismatch,
        e::Refusal::MembershipMismatch,
        e::Refusal::AcceptedMismatch,
        e::Refusal::AnnotationsMismatch,
        e::Refusal::ObservationMismatch,
    ];
    for (coordinate, expected) in errors.into_iter().enumerate() {
        let mut subject = accepted.clone();
        match coordinate {
            0 => subject.header.schema += 1,
            1 => subject.header.family = e::Family::Birth,
            2 => subject.header.subject.as_mut().unwrap().bytes[0] ^= 1,
            3 => subject.header.subject.as_mut().unwrap().address.0[0] ^= 1,
            4 => subject.header.subject.as_mut().unwrap().handle.key.0[0] ^= 1,
            5 => {
                subject
                    .header
                    .provenance
                    .as_mut()
                    .unwrap()
                    .policy_readings
                    .insert("invented".into(), "reading".into());
            }
            6 => subject.header.trace.as_mut().unwrap().requirements.clear(),
            7 => {
                reading::body_mut(&mut subject).rule = e::Rule::Fixed {
                    asset: e::Asset([3; 32]),
                    threshold: e::Amount(0),
                };
            }
            8 => subject.header.evidence.as_mut().unwrap().retrieved.clear(),
            9 => subject.header.assumptions.as_mut().unwrap().clear(),
            10 => {
                reading::body_mut(&mut subject).reading.outcome =
                    e::Outcome::NotHandle(reading::handle(20));
            }
            11 => {
                reading::accepted_mut(&mut subject)
                    .reading
                    .membership
                    .remove(&reading::handle(10));
            }
            12 => {
                reading::accepted_mut(&mut subject)
                    .accepted
                    .remove(&reading::handle(10));
            }
            13 => {
                reading::accepted_mut(&mut subject)
                    .annotations
                    .get_mut(&reading::handle(10))
                    .unwrap()
                    .value = e::Value::Explicit(e::Amount(1020));
            }
            14 => reading::body_mut(&mut subject).epoch = e::Epoch(2),
            _ => unreachable!(),
        }
        assert_eq!(e::validate_report(input, request, accepted), Ok(()));
        assert_eq!(
            e::validate_report(input, request, &subject),
            Err(expected.clone())
        );
        assert_eq!(expected.boundary(), e::Boundary::ReportFraming);
        assert!(
            expected
                .refusal_kind()
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_')
        );
    }
}
fn quantities(
    input: &vectors::keyed_history_reader::Input,
    request: &e::Request,
    accepted: &e::Report,
) {
    assert_eq!(accepted.header.quantities.len(), 12);
    assert_eq!(
        accepted.header.quantities[&e::Dimension::Records].observed,
        e::Measurement::Present {
            value: 2,
            unit: e::Unit::Records
        }
    );
    assert_eq!(
        accepted.header.quantities[&e::Dimension::SubjectBytes].observed,
        e::Measurement::Present {
            value: u64::try_from(input.bytes[&20].len()).unwrap(),
            unit: e::Unit::Bytes
        }
    );
    for dimension in [
        e::Dimension::Operations,
        e::Dimension::StackItems,
        e::Dimension::Weight,
        e::Dimension::NativeWall,
        e::Dimension::NativeCpu,
        e::Dimension::NativeStall,
        e::Dimension::ValidationCharge,
        e::Dimension::DeploymentBound,
        e::Dimension::PackageBound,
    ] {
        assert_eq!(
            accepted.header.quantities[&dimension].observed,
            e::Measurement::Absent(e::Absence::NotObservedHere)
        );
    }
    for (coordinate, expected) in [
        (0, e::Refusal::QuantityMissing(e::Dimension::Records)),
        (1, e::Refusal::QuantityUnit(e::Dimension::Records)),
        (2, e::Refusal::QuantityValue(e::Dimension::Records)),
        (3, e::Refusal::QuantityAvailability(e::Dimension::Records)),
        (
            4,
            e::Refusal::QuantityAvailability(e::Dimension::NativeWall),
        ),
    ] {
        let mut subject = accepted.clone();
        match coordinate {
            0 => {
                subject.header.quantities.remove(&e::Dimension::Records);
            }
            1 => {
                subject
                    .header
                    .quantities
                    .get_mut(&e::Dimension::Records)
                    .unwrap()
                    .observed = e::Measurement::Present {
                    value: 2,
                    unit: e::Unit::Bytes,
                }
            }
            2 => {
                subject
                    .header
                    .quantities
                    .get_mut(&e::Dimension::Records)
                    .unwrap()
                    .observed = e::Measurement::Present {
                    value: 3,
                    unit: e::Unit::Records,
                }
            }
            3 => {
                subject
                    .header
                    .quantities
                    .get_mut(&e::Dimension::Records)
                    .unwrap()
                    .predicted = e::Measurement::Present {
                    value: 2,
                    unit: e::Unit::Records,
                }
            }
            4 => {
                subject
                    .header
                    .quantities
                    .get_mut(&e::Dimension::NativeWall)
                    .unwrap()
                    .observed = e::Measurement::Present {
                    value: 0,
                    unit: e::Unit::Microseconds,
                }
            }
            _ => unreachable!(),
        }
        assert_eq!(e::validate_report(input, request, accepted), Ok(()));
        assert_eq!(e::validate_report(input, request, &subject), Err(expected));
    }
}
fn provenance(input: &vectors::keyed_history_reader::Input, accepted: &e::Request) {
    for field in [
        e::ProvenanceField::Deployment,
        e::ProvenanceField::PolicyVersion,
        e::ProvenanceField::HistorySource,
        e::ProvenanceField::PolicySource,
        e::ProvenanceField::Inclusion,
        e::ProvenanceField::Unspentness,
        e::ProvenanceField::Retrieval,
        e::ProvenanceField::Finality,
    ] {
        let mut subject = accepted.clone();
        let text = match field {
            e::ProvenanceField::Deployment => &mut subject.provenance.deployment.0,
            e::ProvenanceField::PolicyVersion => &mut subject.provenance.policy_version.0,
            e::ProvenanceField::HistorySource => &mut subject.provenance.history_source.0,
            e::ProvenanceField::PolicySource => &mut subject.provenance.policy_source.0,
            e::ProvenanceField::Inclusion => &mut subject.sources.inclusion.0,
            e::ProvenanceField::Unspentness => &mut subject.sources.unspentness.0,
            e::ProvenanceField::Retrieval => &mut subject.sources.retrieval.0,
            e::ProvenanceField::Finality => &mut subject.sources.finality.0,
        };
        text.clear();
        assert!(e::reading_report(input, accepted).is_ok());
        assert_eq!(
            e::reading_report(input, &subject),
            Err(e::Refusal::EmptyProvenance(field))
        );
    }
}

fn trace(trace: &e::Trace) {
    let table = include_str!("../../../../plans/drafts/wave1-bite-1-5-plan.md");
    let mut expected = std::collections::BTreeSet::new();
    for requirement in &trace.requirements {
        let prefix = format!("| {}. ", requirement.0);
        let row = table.lines().find(|row| row.starts_with(&prefix)).unwrap();
        for fragment in row.split("[A-").skip(1) {
            expected.insert(fragment.split(']').next().unwrap().to_owned());
        }
    }
    assert_eq!(
        trace
            .paper
            .iter()
            .map(|label| label.0.clone())
            .collect::<std::collections::BTreeSet<_>>(),
        expected
    );
    for label in &trace.paper {
        let file = if label.0 == "sec:kpc:limitations" {
            include_str!("../../../../papers/lineage/sections/10_limitations.tex")
        } else {
            include_str!("../../../../papers/lineage/sections/01_interface.tex")
        };
        assert!(file.contains(&format!("\\label{{{}}}", label.0)));
    }
}
